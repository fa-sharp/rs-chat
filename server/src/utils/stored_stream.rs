use std::{
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use fred::{
    prelude::{Client, KeysInterface},
    types::Expiration,
};
use rocket::futures::Stream;
use uuid::Uuid;

use crate::{
    db::{
        models::{ChatRsMessageMeta, ChatRsMessageRole, NewChatRsMessage},
        services::ChatDbService,
        DbConnection, DbPool,
    },
    provider::{ChatRsToolCall, ChatRsUsage},
    utils::create_provider::ProviderConfigInput,
};

/// A wrapper around the chat assistant stream that intermittently caches output in Redis, and
/// saves the assistant's response to the database at the end of the stream.
pub struct StoredChatRsStream<
    S: Stream<Item = Result<crate::provider::ChatRsStreamChunk, crate::provider::ChatRsError>>,
> {
    inner: Pin<Box<S>>,
    provider_config: Option<ProviderConfigInput>,
    redis_client: Client,
    db_pool: DbPool,
    session_id: Uuid,
    buffer: Vec<String>,
    tool_calls: Option<Vec<ChatRsToolCall>>,
    input_tokens: u32,
    output_tokens: u32,
    cost: Option<f32>,
    last_cache_time: Instant,
}

pub const CACHE_KEY_PREFIX: &str = "chat_session:";
const CACHE_INTERVAL: Duration = Duration::from_secs(1); // cache the response every second

impl<S> StoredChatRsStream<S>
where
    S: Stream<Item = Result<crate::provider::ChatRsStreamChunk, crate::provider::ChatRsError>>,
{
    pub fn new(
        stream: S,
        provider_config: ProviderConfigInput,
        db_pool: DbPool,
        redis_client: Client,
        session_id: Option<Uuid>,
    ) -> Self {
        Self {
            inner: Box::pin(stream),
            provider_config: Some(provider_config),
            db_pool,
            redis_client,
            session_id: session_id.unwrap_or_else(|| Uuid::new_v4()),
            buffer: Vec::new(),
            tool_calls: None,
            input_tokens: 0,
            output_tokens: 0,
            cost: None,
            last_cache_time: Instant::now(),
        }
    }

    pub fn session_id(&self) -> &Uuid {
        &self.session_id
    }

    fn save_response(&mut self, interrupted: Option<bool>) {
        let redis_client = self.redis_client.clone();
        let pool = self.db_pool.clone();
        let session_id = self.session_id.clone();
        let config = self.provider_config.take();
        let content = self.buffer.join("");
        let tool_calls = self.tool_calls.take();
        let usage = Some(ChatRsUsage {
            input_tokens: Some(self.input_tokens),
            output_tokens: Some(self.output_tokens),
            cost: self.cost,
        });
        self.buffer.clear();

        tokio::spawn(async move {
            let Ok(db) = pool.get().await else {
                rocket::error!("Couldn't get connection while saving chat response");
                return;
            };
            if let Err(e) = ChatDbService::new(&mut DbConnection(db))
                .save_message(NewChatRsMessage {
                    role: ChatRsMessageRole::Assistant,
                    content: &content,
                    session_id: &session_id,
                    meta: &ChatRsMessageMeta {
                        provider_config: config,
                        interrupted,
                        usage,
                        tool_calls,
                    },
                })
                .await
            {
                rocket::error!("Failed saving chat response, session {}: {}", session_id, e);
            } else {
                rocket::info!("Saved chat response, session {}", session_id);
            }

            let key = format!("{}{}", CACHE_KEY_PREFIX, session_id);
            let _ = redis_client.del::<(), _>(&key).await;
        });
    }

    fn should_cache(&self) -> bool {
        self.last_cache_time.elapsed() >= CACHE_INTERVAL
    }
}

impl<S> Stream for StoredChatRsStream<S>
where
    S: Stream<Item = Result<crate::provider::ChatRsStreamChunk, crate::provider::ChatRsError>>,
{
    type Item = Result<String, crate::provider::ChatRsError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                // Add text to buffer
                if let Some(text) = &chunk.text {
                    self.buffer.push(text.clone());
                }

                // Record tool calls
                if let Some(tool_calls) = chunk.tool_calls {
                    self.tool_calls.get_or_insert_default().extend(tool_calls);
                }

                // Record usage
                if let Some(usage) = chunk.usage {
                    if let Some(input_tokens) = usage.input_tokens {
                        self.input_tokens = input_tokens;
                    }
                    if let Some(output_tokens) = usage.output_tokens {
                        self.output_tokens = output_tokens;
                    }
                    if let Some(cost) = usage.cost {
                        self.cost = Some(cost);
                    }
                }

                // Check if we should cache
                if self.should_cache() {
                    let redis_client = self.redis_client.clone();
                    let session_id = self.session_id.clone();
                    let content = self.buffer.join("");

                    // Spawn async task to cache
                    tokio::spawn(async move {
                        let key = format!("{}{}", CACHE_KEY_PREFIX, session_id);
                        rocket::debug!("Caching chat session {}", session_id);
                        if let Err(e) = redis_client
                            .set::<(), _, _>(
                                &key,
                                &content,
                                Some(Expiration::EX(3600)),
                                None,
                                false,
                            )
                            .await
                        {
                            rocket::error!("Redis cache error: {}", e);
                        }
                    });

                    self.last_cache_time = Instant::now();
                }

                if let Some(text) = chunk.text {
                    Poll::Ready(Some(Ok(text)))
                } else {
                    self.poll_next(cx)
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                // Stream ended, flush final buffer
                println!("Final buffer content: {:?}", self.buffer);
                println!("Tool calls: {:?}", self.tool_calls);
                if !self.buffer.is_empty() || self.tool_calls.is_some() {
                    self.save_response(None);
                }
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S> Drop for StoredChatRsStream<S>
where
    S: Stream<Item = Result<crate::provider::ChatRsStreamChunk, crate::provider::ChatRsError>>,
{
    /// Stream was interrupted. Save response and mark as interrupted
    fn drop(&mut self) {
        if !self.buffer.is_empty() || self.tool_calls.is_some() {
            self.save_response(Some(true));
        }
    }
}
