//! Storage engines

mod local;
mod s3;

pub use local::LocalStorage;
pub use s3::{S3Config, S3Storage};
