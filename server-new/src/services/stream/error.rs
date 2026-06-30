/// Errors that can occur during streaming
#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    #[error("Client streaming error: {0}")]
    Tinistream(#[from] super::tinistream::TiniError),
    #[error("Websocket error: {0}")]
    Websocket(#[from] reqwest_websocket::Error),
}
