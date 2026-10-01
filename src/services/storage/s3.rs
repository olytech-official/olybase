use s3::bucket::Bucket;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("S3 error: {0}")]
    S3Error(String),
}

#[derive(Clone)]
pub struct S3Storage {
    pub client: Bucket,
}

impl S3Storage {
    pub async fn store(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), StorageError> {
        self.client
            .put_object_with_content_type(key, &bytes, content_type)
            .await
            .map_err(|e| StorageError::S3Error(e.to_string()))?;

        Ok(())
    }

    pub async fn get(
        &self,
        key: &str,
    ) -> Result<(String, Vec<u8>), StorageError> {
        let response = self
            .client
            .get_object(key)
            .await
            .map_err(|e| StorageError::S3Error(e.to_string()))?;

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
        self.client
            .delete_object(key)
            .await
            .map_err(|e| StorageError::S3Error(e.to_string()))?;

        Ok(())
    }
}