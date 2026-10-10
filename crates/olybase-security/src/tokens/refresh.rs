use std::net::IpAddr;
use std::time::Duration;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{DateTime, Utc};
use rand::distr::Alphanumeric;
use rand::{RngExt};
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

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

pub struct RefreshTokenRecord<U> {
    pub id: Uuid,
    pub user_id: U,
    pub token_hash: [u8; 32],
    pub family_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub user_agent: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub struct IssuedRefreshToken<U = Uuid> {
    pub plaintext_token: String,
    pub record: RefreshTokenRecord<U>,
}

#[derive(Clone)]
pub struct RefreshTokenService {
    ttl: Duration,
    cookie_access_path: String,
}

impl RefreshTokenService {
    pub fn new(ttl: Duration, cookie_access_path: String) -> Self {
        Self {
            ttl,
            cookie_access_path
        }
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

    pub fn issue<U: Clone>(
        &self,
        user_id: U,
        user_agent: Option<String>,
        ip_address: Option<IpAddr>,
    ) -> Result<IssuedRefreshToken<U>, RefreshError> {
        let plaintext_token = Self::generate_token();
        let token_hash = Self::hash_token(&plaintext_token);
        let expires_at = Utc::now() + chrono::Duration::from_std(self.ttl).map_err(|_| RefreshError::TimestampConversion)?;

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

    pub fn rotate<U: Clone>(
        &self,
        existing: &RefreshTokenRecord<U>,
        is_already_used: bool,
        user_agent: Option<String>,
        ip_address: Option<IpAddr>,
    ) -> Result<IssuedRefreshToken<U>, RefreshError> {
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
        let expires_at = Utc::now() + chrono::Duration::from_std(self.ttl).map_err(|_| RefreshError::TimestampConversion)?;

        let record = RefreshTokenRecord {
            id: Uuid::now_v7(),
            user_id: existing.user_id.clone(),
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

    pub fn create_cookie<U>(&self, token: &IssuedRefreshToken<U>) -> Result<Cookie<'static>, RefreshError> {
        let exp = OffsetDateTime::from_unix_timestamp(token.record.expires_at.timestamp())
            .map_err(|_| RefreshError::TimestampConversion)?;

        Ok(Cookie::build(("refresh_token", token.plaintext_token.clone()))
            .path(self.cookie_access_path.clone())
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .expires(exp)
            .build())
    }

    pub fn create_logout_cookie(&self) -> Cookie<'static> {
        Cookie::build(("refresh_token", ""))
            .path(self.cookie_access_path.clone())
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .expires(OffsetDateTime::UNIX_EPOCH)
            .build()
    }
}