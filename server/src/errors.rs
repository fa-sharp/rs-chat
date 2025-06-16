use rocket::{
    catch, catchers,
    response::{self, Responder},
    serde::json::Json,
    Catcher, Request,
};
use rocket_okapi::response::OpenApiResponderInner;
use schemars::JsonSchema;

use crate::provider::ChatRsError;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    Db(#[from] diesel::result::Error),
    #[error("Authentication error: {0}")]
    Authentication(String),
    #[error("Redis error: {0}")]
    Redis(#[from] fred::error::Error),
    #[error(transparent)]
    Chat(#[from] ChatRsError),
}

#[derive(Debug, JsonSchema, serde::Serialize)]
struct Message {
    message: String,
}
impl Message {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Responder)]
enum ApiErrorResponse {
    #[response(status = 400, content_type = "json")]
    BadRequest(Json<Message>),
    #[response(status = 401, content_type = "json")]
    Unauthorized(Json<Message>),
    #[response(status = 404, content_type = "json")]
    NotFound(Json<Message>),
    #[response(status = 500, content_type = "json")]
    Server(Json<Message>),
}

/// API error response handling
impl<'r, 'o: 'r> response::Responder<'r, 'o> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
        rocket::info!("API error: {:?}", self);
        match self {
            ApiError::Authentication(error) => {
                ApiErrorResponse::Unauthorized(Json(Message::new(&error))).respond_to(req)
            }
            ApiError::Db(error) => match error {
                diesel::result::Error::DatabaseError(kind, _info) => ApiErrorResponse::Server(
                    Json(Message::new(&format!("Database error: {:?}", kind))),
                )
                .respond_to(req),
                diesel::result::Error::NotFound => {
                    ApiErrorResponse::NotFound(Json(Message::new("Not found!"))).respond_to(req)
                }
                _ => ApiErrorResponse::Server(Json(Message::new("Server error!"))).respond_to(req),
            },
            ApiError::Chat(error) => {
                ApiErrorResponse::BadRequest(Json(Message::new(&format!("Error: {}", error))))
                    .respond_to(req)
            }
            _ => ApiErrorResponse::Server(Json(Message::new("Server error!"))).respond_to(req),
        }
    }
}

/// Default JSON catchers for request errors.
pub fn get_catchers() -> Vec<Catcher> {
    catchers![
        bad_request,
        unauthorized,
        unprocessable_entity,
        not_found,
        server_error
    ]
}
#[catch(400)]
fn bad_request(_req: &Request) -> ApiErrorResponse {
    ApiErrorResponse::BadRequest(Json(Message::new("Bad request")))
}
#[catch(401)]
fn unauthorized(_req: &Request) -> ApiErrorResponse {
    ApiErrorResponse::Unauthorized(Json(Message::new("Unauthorized!")))
}
#[catch(404)]
fn not_found(_req: &Request) -> ApiErrorResponse {
    ApiErrorResponse::NotFound(Json(Message::new("Not found!")))
}
#[catch(422)]
fn unprocessable_entity(_req: &Request) -> ApiErrorResponse {
    ApiErrorResponse::BadRequest(Json(Message::new("Incorrectly formatted")))
}
#[catch(500)]
fn server_error(_req: &Request) -> ApiErrorResponse {
    ApiErrorResponse::Server(Json(Message::new("Server error!")))
}

/// OpenAPI specification for API error responses
impl OpenApiResponderInner for ApiError {
    fn responses(
        gen: &mut rocket_okapi::r#gen::OpenApiGenerator,
    ) -> rocket_okapi::Result<rocket_okapi::okapi::openapi3::Responses> {
        use rocket_okapi::okapi::openapi3::{
            MediaType, RefOr, Response as OpenApiResponse, Responses,
        };

        let mut responses = schemars::Map::new();
        let mut content = schemars::Map::new();
        content.insert(
            "application/json".to_string(),
            MediaType {
                schema: Some(gen.json_schema::<Message>()),
                ..Default::default()
            },
        );
        let response_data = vec![
            ("400", "Bad request"),
            ("401", "Authentication error"),
            ("404", "Not found"),
            ("422", "Incorrectly formatted"),
            ("500", "Internal error"),
        ];
        for (status, description) in response_data {
            responses.insert(
                status.to_string(),
                RefOr::Object(OpenApiResponse {
                    description: description.to_string(),
                    content: content.clone(),
                    ..Default::default()
                }),
            );
        }
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}
