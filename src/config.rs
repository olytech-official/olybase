#[cfg(feature = "s3")]
use s3::Bucket;
#[cfg(feature = "s3")]
use s3::creds::Credentials;
#[cfg(feature = "s3")]
use s3::Region;
use thiserror::Error;
use serde::Deserialize;
#[cfg(feature = "sqlx-postgres")]
use sqlx::PgPool;
#[cfg(feature = "sqlx-sqlite")]
use sqlx::SqlitePool;
#[cfg(feature = "jwt")]
use crate::services::security::tokens::jwt::JwtError;

fn default_port() -> u16 { 8000 }
fn default_cors_origin() -> String { "http://localhost:3000".into() }
#[cfg(feature = "tls")]
fn default_use_https() -> bool { false }

#[cfg(feature = "rate-limit")]
fn default_rps() -> u64 { 2 }
#[cfg(feature = "rate-limit")]
fn default_burst() -> u32 { 5 }

#[cfg(feature = "jwt")]
fn default_jwt_exp() -> u64 { 3600 }

#[cfg(feature = "s3")]
fn default_s3_region() -> String { "us-east-1".into() }

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load environment variables: {0}")]
    Envy(#[from] envy::Error),

    #[error("TLS is enabled (USE_HTTPS=true) but certificate or key path is missing")]
    MissingTlsPaths,
    
    #[cfg(feature = "jwt")]
    #[error("JWT key paths are missing and auto-generation feature 'jwt-autogen' is disabled")]
    MissingJwtPaths,

    #[cfg(feature = "jwt")]
    #[error("JWT service error: {0}")]
    JwtService(#[from] JwtError),

    #[cfg(any(feature = "sqlx-postgres", feature = "sqlx-sqlite"))]
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[cfg(feature = "s3")]
    #[error("S3 bucket error: {0}")]
    S3(#[from] s3::error::S3Error),

    #[cfg(feature = "s3")]
    #[error("S3 credentials error: {0}")]
    Credentials(#[from] s3::creds::error::CredentialsError),
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_cors_origin")]
    pub cors_origin: String,

    #[cfg(feature = "tls")]
    #[serde(default = "default_use_https")]
    pub use_https: bool,
    #[cfg(feature = "tls")]
    pub tls_cert_path: Option<String>,
    #[cfg(feature = "tls")]
    pub tls_key_path: Option<String>,

    #[cfg(feature = "rate-limit")]
    #[serde(default = "default_rps")]
    pub rate_limit_per_second: u64,
    #[cfg(feature = "rate-limit")]
    #[serde(default = "default_burst")]
    pub rate_limit_burst: u32,

    #[cfg(feature = "private_cookie")]
    pub cookie_secret: String,

    #[cfg(feature = "jwt")]
    #[serde(default = "default_jwt_exp")]
    pub jwt_exp: u64,
    #[cfg(feature = "jwt")]
    pub jwt_private_key_path: Option<String>,
    #[cfg(feature = "jwt")]
    pub jwt_public_key_path: Option<String>,

    #[cfg(feature = "s3")]
    pub rustfs_access_key: String,
    #[cfg(feature = "s3")]
    pub rustfs_secret_key: String,
    #[cfg(feature = "s3")]
    pub rustfs_endpoint: String,
    #[cfg(feature = "s3")]
    #[serde(default = "default_s3_region")]
    pub rustfs_region: String,
    #[cfg(feature = "s3")]
    pub s3_bucket: String,

    #[cfg(any(feature = "sqlx-postgres", feature = "sqlx-sqlite"))]
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = envy::from_env::<Self>()?;

        #[cfg(all(feature = "tls", not(feature = "tls-autogen")))]
        if config.use_https && (config.tls_cert_path.is_none() || config.tls_key_path.is_none()) {
            return Err(ConfigError::MissingTlsPaths);
        }

        #[cfg(all(feature = "jwt", not(feature = "jwt-autogen")))]
        if config.jwt_private_key_path.is_none() || config.jwt_public_key_path.is_none() {
            return Err(ConfigError::MissingJwtPaths);
        }

        Ok(config)
    }

    pub fn load_envs() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();
        Self::from_env()
    }

    #[cfg(feature = "sqlx-postgres")]
    pub async fn build_pg_pool(self) -> Result<PgPool, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(30)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(&self.database_url)
            .await?;

        Ok(pool)
    }

    #[cfg(feature = "sqlx-sqlite")]
    pub async fn load_sqlite_pool(&self) -> Result<SqlitePool, ConfigError> {
        use sqlx::sqlite::SqliteConnectOptions;
        use std::str::FromStr;

        let options = SqliteConnectOptions::from_str(&self.database_url)?
            .create_if_missing(true);

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        Ok(pool)
    }

    #[cfg(feature = "s3")]
    pub fn build_s3_client(&self) -> Result<Bucket, ConfigError> {
        let creds = Credentials::new(
            Some(&self.rustfs_access_key),
            Some(&self.rustfs_secret_key),
            None,
            None,
            None,
        )?;

        let region = Region::Custom {
            region: self.rustfs_region.clone(),
            endpoint: self.rustfs_endpoint.clone(),
        };

        let bucket = Bucket::new(&self.s3_bucket, region, creds)?.with_path_style();
        Ok(*bucket)
    }
}