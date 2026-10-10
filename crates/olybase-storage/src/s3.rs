use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::creds::error::CredentialsError;
use s3::error::S3Error;
use s3::Region;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("S3 bucket error: {0}")]
    S3(#[from] S3Error),

    #[error("S3 credentials error: {0}")]
    Credentials(#[from] CredentialsError),

    #[error("S3 error: {0}")]
    S3StorageError(String),
}

#[derive(Clone)]
pub struct S3Storage {
    pub client: Bucket,
}

impl S3Storage {
    pub fn new(
        endpoint: &str,
        region: &str,
        bucket_name: &str,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Self, StorageError> {
        let creds = Credentials::new(
            Some(access_key),
            Some(secret_key),
            None,
            None,
            None,
        )?;

        let region = Region::Custom {
            region: region.to_string(),
            endpoint: endpoint.to_string(),
        };

        let bucket = Bucket::new(bucket_name, region, creds)?.with_path_style();

        Ok(Self { client: *bucket })
    }

    pub fn from_bucket(bucket: Bucket) -> Self {
        Self { client: bucket }
    }

    pub async fn store(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
    ) -> Result<(), StorageError> {
        self.client
            .put_object_with_content_type(key, bytes, content_type)
            .await?;

        Ok(())
    }

    pub async fn get(
        &self,
        key: &str,
    ) -> Result<(String, Vec<u8>), StorageError> {
        let response = self.client.get_object(key).await?;

        let bytes = response.bytes().to_vec();

        let content_type = response
            .headers()
            .get("content-type")
            .cloned()
            .unwrap_or_else(|| "application/octet-stream".to_string());

        Ok((content_type, bytes))
    }

    pub async fn delete(
        &self,
        key: &str,
    ) -> Result<(), StorageError> {
        self.client.delete_object(key).await?;

        Ok(())
    }
}