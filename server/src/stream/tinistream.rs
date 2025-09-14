use tinistream_client::{types::*, Client, ClientStreamExt, Error};
use uuid::Uuid;

/// A client for interacting with the tinistream API.
pub struct TinistreamClient {
    client: Client,
}

type TiniResult<T> = Result<T, TiniError>;

#[derive(Debug, thiserror::Error)]
#[error("{status} {code}: {message}")]
pub struct TiniError {
    status: u16,
    code: String,
    message: String,
}

impl TinistreamClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn active_chat_streams(&self, prefix: &str) -> TiniResult<Vec<String>> {
        let streams = self
            .client
            .list_streams()
            .pattern(format!("{prefix}*"))
            .send()
            .await?;

        Ok(streams
            .iter()
            .filter_map(|stream| stream.key.strip_prefix(prefix).map(String::from))
            .collect())
    }

    pub async fn chat_stream_exists(user_id: &Uuid, session_id: &Uuid) -> TiniResult<bool> {
        todo!()
    }

    pub async fn chat_stream_start(&self, key: &str) -> TiniResult<CreateStreamResponse> {
        let res = self
            .client
            .create_stream()
            .body(StreamRequest::builder().key(key))
            .send()
            .await?;
        Ok(res.into_inner())
    }

    pub async fn chat_stream_add(
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

    pub async fn chat_stream_cancel(&self, key: &str) -> TiniResult<String> {
        let res = self
            .client
            .cancel_stream()
            .body(StreamRequest::builder().key(key))
            .send()
            .await?;
        Ok(res.into_inner().id)
    }

    pub async fn chat_stream_end(&self, key: &str) -> TiniResult<String> {
        let res = self
            .client
            .end_stream()
            .body(StreamRequest::builder().key(key))
            .send()
            .await?;
        Ok(res.into_inner().id)
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
