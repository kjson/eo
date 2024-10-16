use aws_sdk_s3::Client as S3Client;
use google_cloud_storage::client::{Client as GCSClient, ClientConfig as GCSClientConfig};
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};

use anyhow::{Ok, Result};
use std::path::Path;
use tokio::fs;

pub enum CloudStorage {
    S3(S3Storage),
    GCS(GCSStorage),
}

impl CloudStorage {
    pub async fn download_file(&self, bucket: &str, key: &str, local_path: &Path) -> Result<()> {
        match self {
            CloudStorage::S3(s3) => s3.download_file(bucket, key, local_path).await,
            CloudStorage::GCS(gcs) => gcs.download_file(bucket, key, local_path).await,
        }
    }

    pub async fn upload_file(&self, bucket: &str, key: &str, local_path: &Path) -> Result<()> {
        match self {
            CloudStorage::S3(s3) => s3.upload_file(bucket, key, local_path).await,
            CloudStorage::GCS(gcs) => gcs.upload_file(bucket, key, local_path).await,
        }
    }
}

pub struct S3Storage {
    client: S3Client,
}

impl S3Storage {
    pub async fn new(region: Option<String>) -> Self {
        let config = aws_config::from_env()
            .region(region.map(aws_sdk_s3::Region::new))
            .load()
            .await;

        Self {
            client: S3Client::new(&config),
        }
    }

    pub async fn download_file(&self, bucket: &str, key: &str, local_path: &Path) -> Result<()> {
        let content = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?
            .body
            .collect()
            .await?
            .into_bytes();
        fs::write(local_path, &content).await?;
        Ok(())
    }

    pub async fn upload_file(&self, bucket: &str, key: &str, local_path: &Path) -> Result<()> {
        // Read the modified content from the local file
        let content = fs::read(local_path).await?;

        // Upload the file back to S3
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(content.into())
            .send()
            .await?;

        Ok(())
    }
}

pub struct GCSStorage {
    client: GCSClient,
}

impl GCSStorage {
    pub async fn new(region: Option<String>) -> Self {
        // TODO: use region in client config.

        // TODO: bubble up the error properly.
        let config = GCSClientConfig::default()
            .with_auth()
            .await
            .expect("Failed to create GCS client");

        Self {
            client: GCSClient::new(config),
        }
    }

    pub async fn download_file(&self, bucket: &str, key: &str, local_path: &Path) -> Result<()> {
        let request = GetObjectRequest {
            bucket: bucket.to_string(),
            object: key.to_string(),
            ..Default::default()
        };

        let data = self
            .client
            .download_object(&request, &Range::default())
            .await?;

        fs::write(local_path, data).await?;

        Ok(())
    }

    pub async fn upload_file(&self, bucket: &str, key: &str, local_path: &Path) -> Result<()> {
        let content = fs::read(local_path).await?;

        let upload_type = UploadType::Simple(Media::new(key.to_string()));
        let request = UploadObjectRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        };

        self.client
            .upload_object(&request, content, &upload_type)
            .await?;

        Ok(())
    }
}
