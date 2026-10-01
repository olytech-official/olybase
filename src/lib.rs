pub mod extractors;

pub mod config;
pub mod services;
pub mod controller;

use std::marker::PhantomData;
use std::net::SocketAddr;
#[cfg(all(feature = "ws", feature = "s3"))]
use std::sync::Arc;
use axum::http::header::{InvalidHeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::{Router};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use crate::config::{AppConfig, ConfigError};

#[cfg(feature = "private_cookie")]
use axum_extra::extract::cookie::Key;
#[cfg(feature = "ws")]
use crate::services::gateway::service::GatewayService;
#[cfg(feature = "ws")]
use crate::services::gateway::ws_router::WsRouter;
#[cfg(feature = "s3")]
use crate::services::storage::s3::S3Storage;

#[cfg(feature = "tracing")]
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[derive(Clone)]
pub struct AppState<T> {
    #[cfg(feature = "private_cookie")]
    pub key: Key,
    #[cfg(feature = "ws")]
    pub gateway: Arc<GatewayService>,
    #[cfg(feature = "ws")]
    pub ws_router: WsRouter<T>,
    #[cfg(feature = "s3")]
    pub s3_storage: Arc<S3Storage>,

    pub _marker: PhantomData<fn() -> T>,
}

impl<T> AppState<T> {
    /// Creates an `AppState` using the data provided in `AppConfig`.
    pub async fn new(
        _config: &AppConfig,
        #[cfg(feature = "ws")] gateway: Arc<GatewayService>,
        #[cfg(feature = "ws")] ws_router: WsRouter<T>,
    ) -> Result<Self, ConfigError> {
        #[cfg(feature = "s3")]
        let s3_storage = Arc::new(S3Storage {
            client: _config.build_s3_client()?,
        });

        Ok(Self {
            #[cfg(feature = "private_cookie")]
            key: Key::from(_config.cookie_secret.as_bytes()),
            #[cfg(feature = "ws")]
            gateway,
            #[cfg(feature = "ws")]
            ws_router,
            #[cfg(feature = "s3")]
            s3_storage,

            _marker: PhantomData,
        })
    }
}

pub struct Server {
    pub config: AppConfig,
    cors_layer: Option<CorsLayer>,
    app: Option<Router>,
}

impl Server {
    pub fn new(config: AppConfig) -> Self {
        Server {
            config,
            cors_layer: None,
            app: None,
        }
    }

    /// adds optional `cors` to the router
    pub fn with_cors(mut self) -> Result<Self, InvalidHeaderValue> {
        let origin = self.config.cors_origin.parse::<HeaderValue>()?;

        let cors = CorsLayer::new()
            .allow_origin(origin)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_credentials(true)
            .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

        self.cors_layer = Some(cors);
        Ok(self)
    }

    /// builds the `router`
    pub fn with_router<T>(
        mut self,
        router: Router<AppState<T>>,
        state: AppState<T>
    ) -> Self where
        T: Send + Sync + Clone + 'static,
    {
        let mut app = router;

        if let Some(cors) = self.cors_layer.take() {
            app = app.layer(cors);
        }

        #[cfg(feature = "ws")]
        {
            let gateway = state.gateway.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    gateway.cleanup().await;
                }
            });
        }

        #[cfg(feature = "rate-limit")]
        {
            use tower_governor::governor::GovernorConfigBuilder;
            use tower_governor::GovernorLayer;
            use std::sync::Arc;

            let gov_config = Arc::new(GovernorConfigBuilder::default()
                .per_second(self.config.rate_limit_per_second)
                .burst_size(self.config.rate_limit_burst)
                .finish()
                .expect("Failed to configure rate limiter"));

            app = app.layer(GovernorLayer::new(gov_config));
        }

        #[cfg(feature = "compression")]
        {
            use tower_http::compression::CompressionLayer;
            app = app.layer(CompressionLayer::new());
        }

        #[cfg(feature = "tracing")]
        {
            use axum::http::HeaderName;
            use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
            use tower_http::trace::{DefaultOnResponse, TraceLayer};
            use tracing::Level;

            let x_request_id = HeaderName::from_static("x-request-id");

            app = app
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &axum::http::Request<_>| {
                            let request_id = request
                                .headers()
                                .get("x-request-id")
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or("unknown");

                            tracing::info_span!(
                                "http_request",
                                method = %request.method(),
                                uri = %request.uri(),
                                version = ?request.version(),
                                request_id = %request_id,
                            )
                        })
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
                .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid));
        }

        let app = app.with_state(state);

        self.app = Some(app);
        self
    }

    /// starts the `router`
    pub async fn start_server(self) -> Result<(), Box<dyn std::error::Error>> {
        let Server { config, app, .. } = self;
        let app = app.ok_or("Router not initialized. Did you call .with_router(router, state)?")?;

        let addr: SocketAddr = format!("0.0.0.0:{}", config.port).parse()?;

        #[cfg(feature = "tracing")]
        tracing::info!("Starting server on port {}", config.port);

        #[cfg(feature = "tls")]
        if config.use_https {
            let tls_config = match (
                &config.tls_cert_path,
                &config.tls_key_path
            ) {
                (Some(cert_path), Some(key_path)) => {
                    axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path).await?
                }

                #[cfg(feature = "tls-autogen")]
                (None, None) => {
                    eprintln!("[WARN] No CERTS provided. Generating CERTS via rcgen.");

                    let subject_alt_names = vec!["localhost".to_string()];
                    let cert_key = rcgen::generate_simple_self_signed(subject_alt_names)?;

                    axum_server::tls_rustls::RustlsConfig::from_pem(
                        cert_key.cert.pem().into_bytes(),
                        cert_key.signing_key.serialize_pem().into_bytes(),
                    ).await?
                }

                _ => return Err("Missing TLS cert/key paths. Provide paths or enable 'tls-autogen'".into()),
            };

            println!("Server running on https://{}", addr);

            axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .await?;

            return Ok(());
        }

        let listener = TcpListener::bind(addr).await?;

        println!("Server running on http://{}", addr);
        axum::serve(listener, app).await?;

        Ok(())
    }
}