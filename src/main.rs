use axum::Router;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use olybase::{AppState, Server};
use olybase::config::AppConfig;
use olybase::extractors::user_auth_guard::AuthenticatedUser;

pub async fn index() -> &'static str {
    "Hello, World!"
}

pub async fn protected_route(user: AuthenticatedUser<UserClaims>) -> String {
    format!("Welcome back, {} (ID: {})", user.custom.name, user.id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub name: String,
}

pub fn routes<T>() -> Router<AppState<T>>
where
    T: Send + Sync + Clone + 'static,
{
    Router::new()
        .route("/", get(index))
        .route("/protected", get(protected_route))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;

    let state = AppState::new(&config).await?;

    let router = routes::<UserClaims>();

    Server::new(config)
        .with_cors()?
        .with_router(router, state)
        .start_server()
        .await?;

    Ok(())
}