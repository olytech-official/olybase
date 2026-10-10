use std::fs;
use std::time::Duration;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Failed to read key file at '{0}': {1}")]
    Io(String, std::io::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Invalid timestamp conversion for cookie expiration")]
    TimestampConversion,

    #[error("JWT private and public key paths must both be provided when autogen is disabled")]
    MissingKeyPaths,

    #[error("Failed to auto-generate RSA keypair: {0}")]
    AutogenFailed(String),
}

pub struct ClaimsData<T, S> {
    pub sub: S,
    pub custom: T,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims<T, S> {
    pub sub: S,
    pub iat: usize,
    pub exp: usize,
    #[serde(flatten)]
    pub custom: T,
}

#[derive(Clone)]
pub struct JwtService {
    ttl: Duration,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtService {
    pub fn new(
        jwt_private_key_path: Option<String>,
        jwt_public_key_path: Option<String>,
        ttl: Duration,
    ) -> Result<Self, JwtError> {
        let (private_pem, public_pem) = match (
            &jwt_private_key_path,
            &jwt_public_key_path,
        ) {
            (Some(priv_path), Some(pub_path)) => {
                let priv_pem = fs::read_to_string(priv_path)
                    .map_err(|e| JwtError::Io(priv_path.clone(), e))?;
                let pub_pem = fs::read_to_string(pub_path)
                    .map_err(|e| JwtError::Io(pub_path.clone(), e))?;
                (priv_pem, pub_pem)
            }

            #[cfg(feature = "jwt-autogen")]
            (None, None) => {
                let key_pair = rcgen::KeyPair::generate_for(&rcgen::PKCS_RSA_SHA256)
                    .map_err(|e| JwtError::AutogenFailed(e.to_string()))?;

                (key_pair.serialize_pem(), key_pair.public_key_pem())
            }

            _ => return Err(JwtError::MissingKeyPaths),
        };

        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes())?;
        let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())?;
        let validation = Validation::new(Algorithm::RS256);

        Ok(Self {
            ttl,
            encoding_key,
            decoding_key,
            validation,
        })
    }

    pub fn create<T: Serialize, S: Serialize>(&self, claims_data: ClaimsData<T, S>) -> Result<Cookie<'static>, JwtError> {
        let now = Utc::now();
        let exp = now + chrono::Duration::from_std(self.ttl).map_err(|_| JwtError::TimestampConversion)?;

        let claims = Claims {
            sub: claims_data.sub,
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
            custom: claims_data.custom,
        };

        let header = Header::new(Algorithm::RS256);
        let token = encode(&header, &claims, &self.encoding_key)?;

        let cookie_exp = OffsetDateTime::from_unix_timestamp(exp.timestamp())
            .map_err(|_| JwtError::TimestampConversion)?;

        Ok(Cookie::build(("jwt", token))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .expires(cookie_exp)
            .build())
    }

    pub fn verify<T: DeserializeOwned, S: DeserializeOwned>(&self, token: &str) -> Result<ClaimsData<T, S>, JwtError> {
        let token_data = decode::<Claims<T, S>>(token, &self.decoding_key, &self.validation)?;

        Ok(ClaimsData {
            sub: token_data.claims.sub,
            custom: token_data.claims.custom,
        })
    }
}