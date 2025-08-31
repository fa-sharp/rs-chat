use rocket::{
    async_trait,
    data::{FromData, Outcome, ToByteUnit},
    http::{ContentType, Status},
    outcome::{try_outcome, IntoOutcome},
    Request,
};
use rocket_okapi::request::OpenApiFromData;

use crate::db::models::ChatRsFileType;

const MAX_FILE_SIZE: usize = 4 * 1024 * 1024; // 4 MB

/// Data guard for file uploads
pub struct FileData<'r> {
    pub data: rocket::data::DataStream<'r>,
    pub content_type: &'r ContentType,
    pub file_type: ChatRsFileType,
    pub content_length: usize,
}

#[async_trait]
impl<'r> FromData<'r> for FileData<'r> {
    type Error = &'static str;

    async fn from_data(
        req: &'r Request<'_>,
        mut data: rocket::Data<'r>,
    ) -> Outcome<'r, Self, Self::Error> {
        if data.peek(8).await.is_empty() {
            return Outcome::Error((Status::BadRequest, "No data found"));
        }
        let content_type = try_outcome!(req
            .content_type()
            .or_error((Status::BadRequest, "No content type found")));
        let content_length: usize = try_outcome!(req
            .headers()
            .get_one("Content-Length")
            .map(|s| s.parse().unwrap_or(0))
            .or_error((Status::LengthRequired, "No content length found")));
        if content_length > MAX_FILE_SIZE {
            return Outcome::Error((Status::PayloadTooLarge, "File size exceeds maximum"));
        }

        let file_type = {
            if content_type.is_jpeg()
                || content_type.is_png()
                || content_type.is_webp()
                || content_type.is_bmp()
            {
                ChatRsFileType::Image
            } else if content_type.is_pdf() {
                ChatRsFileType::Pdf
            } else {
                ChatRsFileType::Text
            }
        };

        Outcome::Success(FileData {
            data: data.open(5.mebibytes()),
            file_type,
            content_length,
            content_type,
        })
    }
}

impl<'r> OpenApiFromData<'r> for FileData<'r> {
    fn request_body(
        _gen: &mut rocket_okapi::r#gen::OpenApiGenerator,
    ) -> rocket_okapi::Result<rocket_okapi::okapi::openapi3::RequestBody> {
        Ok(rocket_okapi::okapi::openapi3::RequestBody {
            description: Some("File data".to_string()),
            content: {
                let mut content = schemars::Map::new();
                content.insert(
                    "application/octet-stream".into(),
                    rocket_okapi::okapi::openapi3::MediaType {
                        schema: Some(rocket_okapi::okapi::openapi3::SchemaObject {
                            instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(
                                schemars::schema::InstanceType::String,
                            ))),
                            format: Some("binary".to_string()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                );
                content
            },
            required: true,
            ..Default::default()
        })
    }
}
