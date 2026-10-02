use std::net::IpAddr;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{DateTime, Duration, Utc};
use rand::distr::Alphanumeric;
use rand::{RngExt};
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;
use crate::config::AppConfig;

#[derive(Debug, Error)]
pub enum RefreshError {
    #[error("Refresh token not found")]
    NotFound,

    #[error("Refresh token expired")]
    Expired,

    #[error("Refresh token revoked")]
    Revoked,

    #[error("Refresh token reuse detected")]
    ReuseDetected,

    #[error("Invalid timestamp conversion for cookie expiration")]
    TimestampConversion,
}

pub struct RefreshTokenRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: [u8; 32],
    pub family_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub user_agent: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub struct IssuedRefreshToken {
    pub plaintext_token: String,
    pub record: RefreshTokenRecord,
}

#[derive(Clone)]
pub struct RefreshTokenService {
    exp: u64,
    cookie_path: String,
}

impl RefreshTokenService {
    pub fn new(config: AppConfig) -> Self {
        Self {
            exp: config.refresh_exp,
            cookie_path: config.refresh_path,
        }
    }

    pub fn with_cookie_path(mut self, path: impl Into<String>) -> Self {
        self.cookie_path = path.into();
        self
    }

    fn generate_token() -> String {
        rand::rng()
            .sample_iter(Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    pub fn hash_token(token: &str) -> [u8; 32] {
        Sha256::digest(token.as_bytes()).into()
    }

    pub fn issue(
        &self,
        user_id: Uuid,
        user_agent: Option<String>,
        ip_address: Option<IpAddr>,
    ) -> Result<IssuedRefreshToken, RefreshError> {
        let plaintext_token = Self::generate_token();
        let token_hash = Self::hash_token(&plaintext_token);
        let expires_at = Utc::now() + Duration::seconds(self.exp as i64);

        let record = RefreshTokenRecord {
            id: Uuid::now_v7(),
            user_id,
            token_hash,
            family_id: Uuid::now_v7(),
            parent_id: None,
            user_agent,
            ip_address,
            expires_at,
            revoked_at: None,
        };

        Ok(IssuedRefreshToken {
            plaintext_token,
            record,
        })
    }

    pub fn rotate(
        &self,
        existing: &RefreshTokenRecord,
        is_already_used: bool,
        user_agent: Option<String>,
        ip_address: Option<IpAddr>,
    ) -> Result<IssuedRefreshToken, RefreshError> {
        if is_already_used {
            return Err(RefreshError::ReuseDetected);
        }

        if existing.revoked_at.is_some() {
            return Err(RefreshError::Revoked);
        }

        if existing.expires_at < Utc::now() {
            return Err(RefreshError::Expired);
        }

        let plaintext_token = Self::generate_token();
        let token_hash = Self::hash_token(&plaintext_token);
        let expires_at = Utc::now() + Duration::seconds(self.exp as i64);

        let record = RefreshTokenRecord {
            id: Uuid::now_v7(),
            user_id: existing.user_id,
            token_hash,
            family_id: existing.family_id,
            parent_id: Some(existing.id),
            user_agent,
            ip_address,
            expires_at,
            revoked_at: None,
        };

        Ok(IssuedRefreshToken {
            plaintext_token,
            record
        })
    }

    pub fn create_cookie(&self, token: &IssuedRefreshToken) -> Result<Cookie<'static>, RefreshError> {
        let exp = OffsetDateTime::from_unix_timestamp(token.record.expires_at.timestamp())
            .map_err(|_| RefreshError::TimestampConversion)?;

        Ok(Cookie::build(("refresh_token", token.plaintext_token.clone()))
            .path(self.cookie_path.clone())
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .expires(exp)
            .build())
    }

    pub fn create_logout_cookie(&self) -> Cookie<'static> {
        Cookie::build(("refresh_token", ""))
            .path(self.cookie_path.clone())
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .expires(OffsetDateTime::UNIX_EPOCH)
            .build()
    }
}