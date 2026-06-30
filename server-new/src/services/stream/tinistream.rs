//! Client for `tinistream` to handle streaming responses

use reqwest_websocket::{Upgrade, WebSocket};
use tinistream_client::{Client, ClientEventsExt, ClientInfo, ClientStreamExt, Error, types::*};

/// A client for interacting with the `tinistream` API.
#[derive(Debug, Clone)]
pub struct TinistreamClient {
    client: Client,
}

/// Result type for tinistream API operations.
pub type TiniResult<T> = Result<T, TiniError>;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct TiniError {
    pub status: u16,
    pub code: String,
    pub message: String,
}

impl TinistreamClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Returns a list of keys with the given prefix that have an active stream.
    pub async fn active_streams(&self, prefix: &str) -> TiniResult<Vec<StreamInfo>> {
        let streams = self
            .client
            .list_streams()
            .pattern(format!("{prefix}*"))
            .send()
            .await?
            .into_inner();
        Ok(streams)
    }

    /// Returns whether an active chat stream exists for the given key.
    pub async fn stream_exists(&self, key: &str) -> TiniResult<bool> {
        match self.client.get_stream_info().key(key).send().await {
            Ok(_) => Ok(true),
            Err(err) => match err.status() {
                Some(reqwest::StatusCode::NOT_FOUND) => Ok(false),
                _ => Err(err.into()),
            },
        }
    }

    /// Returns info about a chat stream at the given key.
    pub async fn stream_info(&self, key: &str) -> TiniResult<Option<StreamInfo>> {
        let info = self.client.get_stream_info().key(key).send().await?;
        Ok(Some(info.into_inner()))
    }

    /// Start the chat stream and get the client URL and access token
    pub async fn stream_start(&self, key: &str) -> TiniResult<StreamAccessResponse> {
        let res = self
            .client
            .create_stream()
            .body(StreamRequest::builder().key(key))
            .send()
            .await?;
        Ok(res.into_inner())
    }

    /// Get URL and token for a client to access a stream
    pub async fn stream_connect(&self, key: &str) -> TiniResult<StreamAccessResponse> {
        let res = self
            .client
            .create_token()
            .body(StreamRequest::builder().key(key))
            .send()
            .await?;
        Ok(res.into_inner())
    }

    /// Get a WebSocket connection to write to a stream
    pub async fn stream_writer_ws(&self, key: &str) -> Result<WebSocket, reqwest_websocket::Error> {
        let http_client = self.client.client();
        let res = http_client
            .get(format!("{}/api/event/add/ws-stream", self.client.baseurl()))
            .query(&[("key", key)])
            .upgrade()
            .send()
            .await?;
        res.into_websocket().await
    }

    pub async fn stream_add(
        &self,
        key: &str,
        events: Vec<builder::AddEvent>,
    ) -> TiniResult<Vec<String>> {
        let events = events
            .into_iter()
            .map(|event| event.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        let res = self
            .client
            .add_events()
            .body(AddEventsRequest::builder().key(key).events(events))
            .send()
            .await?;
        Ok(res.into_inner().ids)
    }

    /// Cancel a stream
    pub async fn stream_cancel(&self, key: &str) -> TiniResult<StreamStatus> {
        let res = self
            .client
            .cancel_stream()
            .body(StreamRequest::builder().key(key))
            .send()
            .await?;
        Ok(res.into_inner().status)
    }

    /// Signal the end of a stream
    pub async fn stream_end(&self, key: &str) -> TiniResult<StreamStatus> {
        let res = self
            .client
            .end_stream()
            .body(StreamRequest::builder().key(key))
            .send()
            .await?;
        Ok(res.into_inner().status)
    }
}

impl From<Error<ErrorMessage>> for TiniError {
    fn from(value: Error<ErrorMessage>) -> Self {
        match value {
            Error::ErrorResponse(res) => {
                let status = res.status().as_u16();
                let res = res.into_inner();
                TiniError {
                    status,
                    code: res.code,
                    message: res.message,
                }
            }
            res => TiniError {
                status: res.status().map_or(500, |s| s.as_u16()),
                code: "unexpected".to_owned(),
                message: res.to_string(),
            },
        }
    }
}

impl From<error::ConversionError> for TiniError {
    fn from(value: error::ConversionError) -> Self {
        TiniError {
            status: 400,
            code: "invalid_event".to_owned(),
            message: value.to_string(),
        }
    }
}
