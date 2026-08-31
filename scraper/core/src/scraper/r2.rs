use aws_sdk_s3::{Client, Config};
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;

use crate::errors::{AppError, Result};

#[derive(Clone)]
pub struct R2Client {
    client:     Client,
    bucket:     String,
    public_url: String,
}

impl R2Client {
    pub fn from_env() -> Result<Self> {
        let access_key = std::env::var("R2_ACCESS_KEY_ID")
            .map_err(|_| AppError::Env("R2_ACCESS_KEY_ID not set".into()))?;
        let secret_key = std::env::var("R2_SECRET_ACCESS_KEY")
            .map_err(|_| AppError::Env("R2_SECRET_ACCESS_KEY not set".into()))?;
        let bucket = std::env::var("R2_BUCKET_NAME")
            .map_err(|_| AppError::Env("R2_BUCKET_NAME not set".into()))?;
        let endpoint = std::env::var("R2_ENDPOINT")
            .map_err(|_| AppError::Env("R2_ENDPOINT not set".into()))?;
        let public_url = std::env::var("R2_PUBLIC_URL")
            .map_err(|_| AppError::Env("R2_PUBLIC_URL not set".into()))?;

        let creds = Credentials::new(access_key, secret_key, None, None, "r2");
        let config = Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(creds)
            .endpoint_url(endpoint)
            .region(Region::new("auto"))
            .force_path_style(true)
            .build();

        Ok(Self {
            client:     Client::from_conf(config),
            bucket,
            public_url: public_url.trim_end_matches('/').to_string(),
        })
    }

    /// Uploads `bytes` to `key` and returns the full public URL.
    pub async fn upload(&self, key: &str, bytes: Vec<u8>) -> Result<String> {
        let content_type = if key.ends_with(".webp") {
            "image/webp"
        } else if key.ends_with(".jpg") || key.ends_with(".jpeg") {
            "image/jpeg"
        } else if key.ends_with(".png") {
            "image/png"
        } else {
            "application/octet-stream"
        };

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(bytes))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::Scrape(format!("R2 upload '{}': {}", key, e)))?;

        Ok(format!("{}/{}", self.public_url, key))
    }

    /// Deletes `key`. Used when a repaired upload replaces a file under a new
    /// (differently-named) key and the stale object needs cleaning up.
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::Scrape(format!("R2 delete '{}': {}", key, e)))?;
        Ok(())
    }
}
