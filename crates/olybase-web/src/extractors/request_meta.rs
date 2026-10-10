use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::{header, request::Parts},
};
use std::net::{IpAddr, SocketAddr};

pub struct RequestMeta {
    pub user_agent: Option<String>,
    pub ip: Option<IpAddr>,
}

impl<S> FromRequestParts<S> for RequestMeta
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user_agent = parts
            .headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let header_ip = parts
            .headers
            .get("cf-connecting-ip")
            .or_else(|| parts.headers.get("x-forwarded-for"))
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok());

        let ip = match header_ip {
            Some(ip) => Some(ip),
            None => {
                ConnectInfo::<SocketAddr>::from_request_parts(parts, state)
                    .await
                    .map(|ConnectInfo(addr)| addr.ip())
                    .ok()
            }
        };

        Ok(RequestMeta { user_agent, ip })
    }
}