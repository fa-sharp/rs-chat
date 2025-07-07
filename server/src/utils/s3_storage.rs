use aws_sdk_s3::{
    config::{AppName, Credentials, Region},
    operation::head_bucket::HeadBucketError,
    Client, Config,
};

pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    pub fn new(
        endpoint_url: &str,
        access_key: &str,
        secret_key: &str,
        region: String,
        bucket: String,
    ) -> Self {
        let credentials = Credentials::builder()
            .access_key_id(access_key)
            .secret_access_key(secret_key)
            .provider_name("Custom S3 Provider")
            .build();
        let config = Config::builder()
            .endpoint_url(endpoint_url)
            .credentials_provider(credentials)
            .app_name(AppName::new("rs-chat").expect("Should be valid name"))
            .region(Region::new(region))
            .build();
        let client = Client::from_conf(config);

        Self { client, bucket }
    }

    pub async fn check_bucket(&self) -> Result<bool, HeadBucketError> {
        match self.client.head_bucket().bucket(&self.bucket).send().await {
            Ok(_) => Ok(true),
            Err(err) => match err.into_service_error() {
                HeadBucketError::NotFound(_) => Ok(false),
                err @ _ => return Err(err),
            },
        }
    }
}
