/// Streaming infrastructure errors
#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    #[error("tinistream error: {0}")]
    Tinistream(#[from] super::tinistream::TiniError),
    #[error("websocket error: {0}")]
    Websocket(#[from] reqwest_websocket::Error),
}
