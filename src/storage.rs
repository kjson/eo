use aws_sdk_s3::Client;

use anyhow::Result;
use std::fs;
use std::path::Path;

pub enum CloudStorage {
    S3(S3Storage),
}

impl CloudStorage {
    pub async fn download_file(&self, bucket: &str, key: &str, local_path: &Path) -> Result<()> {
        match self {
            CloudStorage::S3(s3) => s3.download_file(bucket, key, local_path).await,
        }
    }

    pub async fn upload_file(&self, bucket: &str, key: &str, local_path: &Path) -> Result<()> {
        match self {
            CloudStorage::S3(s3) => s3.upload_file(bucket, key, local_path).await,
        }
    }
}

pub struct S3Storage {
    client: Client,
}

impl S3Storage {
    pub async fn new(region: Option<String>) -> Self {
        let config = aws_config::from_env()
            .region(region.map(aws_sdk_s3::Region::new))
            .load()
            .await;

        Self {
            client: Client::new(&config),
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
        fs::write(local_path, &content)?;
        Ok(())
    }

    pub async fn upload_file(&self, bucket: &str, key: &str, local_path: &Path) -> Result<()> {
        // Read the modified content from the local file
        let content = fs::read(local_path)?;

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
