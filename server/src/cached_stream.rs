use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use fred::prelude::{Client, KeysInterface};
use fred::types::Expiration;
use rocket::futures::Stream;
use uuid::Uuid;

pub struct CachedStream<S> {
    inner: Pin<Box<S>>,
    redis_client: Client,
    session_id: String,
    buffer: Vec<String>,
    last_cache_time: Instant,
    cache_interval: Duration,
}

impl<S> CachedStream<S>
where
    S: Stream<Item = Result<String, crate::provider::ChatRsError>>,
{
    pub fn new(stream: S, redis_client: Client, session_id: Option<String>) -> Self {
        Self {
            inner: Box::pin(stream),
            redis_client,
            session_id: session_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            buffer: Vec::new(),
            last_cache_time: Instant::now(),
            cache_interval: Duration::from_secs(4),
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    // async fn flush_to_redis(&mut self) -> Result<(), fred::prelude::Error> {
    //     if self.buffer.is_empty() {
    //         return Ok(());
    //     }

    //     let key = format!("chat_session:{}", self.session_id);
    //     let content = self.buffer.join("");

    //     // Store with 1 hour expiration
    //     let _: () = self
    //         .redis_client
    //         .set(&key, &content, Some(Expiration::EX(3600)), None, false)
    //         .await?;

    //     println!(
    //         "Cached {} characters to Redis for session {}",
    //         content.len(),
    //         self.session_id
    //     );

    //     Ok(())
    // }

    fn should_cache(&self) -> bool {
        self.last_cache_time.elapsed() >= self.cache_interval
    }
}

impl<S> Stream for CachedStream<S>
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
                        let key = format!("chat_session:{}", session_id);
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
                            eprintln!("Redis cache error: {}", e);
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
                    let redis_client = self.redis_client.clone();
                    let session_id = self.session_id.clone();
                    let content = self.buffer.join("");
                    self.buffer.clear();

                    tokio::spawn(async move {
                        let key = format!("chat_session:{}", session_id);
                        rocket::info!("Final cache for chat session {}", session_id);
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
                            eprintln!("Redis cache error: {}", e);
                        }
                    });
                }
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S> Drop for CachedStream<S> {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            let redis_client = self.redis_client.clone();
            let session_id = self.session_id.clone();
            let content = self.buffer.join("");

            tokio::spawn(async move {
                let key = format!("chat_session:{}", session_id);
                rocket::info!(
                    "Stream dropped, final cache for chat session {}",
                    session_id
                );
                if let Err(e) = redis_client
                    .set::<(), _, _>(&key, &content, Some(Expiration::EX(3600)), None, false)
                    .await
                {
                    eprintln!("Redis cache error: {}", e);
                }
            });
        }
    }
}
