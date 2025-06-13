use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use fred::prelude::{Client, KeysInterface};
use fred::types::Expiration;
use rocket::futures::Stream;
use uuid::Uuid;

use crate::db::models::{ChatRsMessageRole, NewChatRsMessage};
use crate::db::services::chat::ChatDbService;
use crate::db::{DbConnection, DbPool};

/// A wrapper around the chat assistant stream that intermittently caches output in Redis, and
/// saves the assistant's response to the database at the end of the stream.
pub struct StoredChatRsStream<S: Stream<Item = Result<String, crate::provider::ChatRsError>>> {
    inner: Pin<Box<S>>,
    redis_client: Client,
    db_pool: DbPool,
    session_id: Uuid,
    buffer: Vec<String>,
    last_cache_time: Instant,
}

const CACHE_KEY_PREFIX: &str = "chat_session:";
const CACHE_INTERVAL: Duration = Duration::from_secs(4);

impl<S> StoredChatRsStream<S>
where
    S: Stream<Item = Result<String, crate::provider::ChatRsError>>,
{
    pub fn new(stream: S, db_pool: DbPool, redis_client: Client, session_id: Option<Uuid>) -> Self {
        Self {
            inner: Box::pin(stream),
            db_pool,
            redis_client,
            session_id: session_id.unwrap_or_else(|| Uuid::new_v4()),
            buffer: Vec::new(),
            last_cache_time: Instant::now(),
        }
    }

    pub fn session_id(&self) -> &Uuid {
        &self.session_id
    }

    fn save_response(&mut self) {
        let redis_client = self.redis_client.clone();
        let pool = self.db_pool.clone();
        let session_id = self.session_id.clone();
        let content = self.buffer.join("");
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
    S: Stream<Item = Result<String, crate::provider::ChatRsError>>,
{
    type Item = Result<String, crate::provider::ChatRsError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(message))) => {
                // Add to buffer
                self.buffer.push(message.clone());

                // Check if we should cache
                if self.should_cache() {
                    let redis_client = self.redis_client.clone();
                    let session_id = self.session_id.clone();
                    let content = self.buffer.join("");

                    // Spawn async task to cache
                    tokio::spawn(async move {
                        let key = format!("{}{}", CACHE_KEY_PREFIX, session_id);
                        rocket::info!("Caching chat session {}", session_id);
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

                Poll::Ready(Some(Ok(message)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                // Stream ended, flush final buffer
                if !self.buffer.is_empty() {
                    self.save_response();
                }
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S> Drop for StoredChatRsStream<S>
where
    S: Stream<Item = Result<String, crate::provider::ChatRsError>>,
{
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            self.save_response();
        }
    }
}
