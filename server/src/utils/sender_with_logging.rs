use tokio::sync::mpsc::{error::SendError, Sender};

/// A sender wrapper that sends messages to a primary and a log collector
/// channel, while preserving the primary channel's closure semantics.
#[derive(Debug, Clone)]
pub struct SenderWithLogging<T> {
    primary_tx: Sender<T>,
    log_tx: Sender<T>,
}

impl<T: Clone> SenderWithLogging<T> {
    pub fn new(primary_sender: Sender<T>, log_sender: Sender<T>) -> Self {
        Self {
            primary_tx: primary_sender,
            log_tx: log_sender,
        }
    }

    pub async fn send(&self, chunk: T) -> Result<(), SendError<T>> {
        // Send to log collector first
        let _ = self.log_tx.send(chunk.clone()).await;

        // Send to primary (for streaming) - return this result
        // If this fails, it means the client disconnected
        self.primary_tx.send(chunk).await
    }

    /// Returns future that completes when the primary channel is closed
    pub async fn closed(&self) {
        self.primary_tx.closed().await
    }

    /// Returns whether the primary channel is closed
    pub fn is_closed(&self) -> bool {
        self.primary_tx.is_closed()
    }
}
