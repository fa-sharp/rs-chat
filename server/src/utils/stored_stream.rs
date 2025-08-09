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
        models::{
            AssistantMeta, ChatRsMessageMeta, ChatRsMessageRole, ChatRsToolCall, NewChatRsMessage,
        },
        services::ChatDbService,
        DbConnection, DbPool,
    },
    provider::{LlmApiProviderSharedOptions, LlmUsage},
};

/// A wrapper around the chat assistant stream that intermittently caches output in Redis, and
/// saves the assistant's response to the database at the end of the stream.
pub struct StoredChatRsStream<
    S: Stream<Item = Result<crate::provider::LlmStreamChunk, crate::provider::LlmError>>,
> {
    inner: Pin<Box<S>>,
    provider_id: i32,
    provider_options: Option<LlmApiProviderSharedOptions>,
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
    S: Stream<Item = Result<crate::provider::LlmStreamChunk, crate::provider::LlmError>>,
{
    pub fn new(
        stream: S,
        provider_id: i32,
        provider_options: LlmApiProviderSharedOptions,
        db_pool: DbPool,
        redis_client: Client,
        session_id: Option<Uuid>,
    ) -> Self {
        Self {
            inner: Box::pin(stream),
            provider_id,
            provider_options: Some(provider_options),
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
        let provider_id = self.provider_id.clone();
        let provider_options = self.provider_options.take();
        let content = self.buffer.join("");
        let tool_calls = self.tool_calls.take();
        let usage = Some(LlmUsage {
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
                    meta: ChatRsMessageMeta {
                        assistant: Some(AssistantMeta {
                            provider_id,
                            provider_options,
                            partial: interrupted,
                            usage,
                            tool_calls,
                        }),
                        ..Default::default()
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
    S: Stream<Item = Result<crate::provider::LlmStreamChunk, crate::provider::LlmError>>,
{
    type Item = Result<String, crate::provider::LlmError>;

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
    S: Stream<Item = Result<crate::provider::LlmStreamChunk, crate::provider::LlmError>>,
{
    /// Stream was interrupted. Save response and mark as interrupted
    fn drop(&mut self) {
        if !self.buffer.is_empty() || self.tool_calls.is_some() {
            self.save_response(Some(true));
        }
    }
}
