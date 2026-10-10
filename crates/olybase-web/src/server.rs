use std::net::{SocketAddr};
use tokio::net::TcpListener;
use axum::extract::State;
use axum::http::{HeaderValue, Method};
use axum::http::header::{InvalidHeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::Router;
use tower_http::cors::CorsLayer;

pub struct ServerOptions {
    pub host: String,
    pub port: u16,

    #[cfg(feature = "rate-limit")]
    pub rate_limit_per_second: u64,
    #[cfg(feature = "rate-limit")]
    pub rate_limit_burst: u32,

    #[cfg(feature = "tls")]
    pub use_https: bool,
    #[cfg(feature = "tls")]
    pub tls_cert_path: Option<String>,
    #[cfg(feature = "tls")]
    pub tls_key_path: Option<String>,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,

            #[cfg(feature = "rate-limit")]
            rate_limit_per_second: 10,
            #[cfg(feature = "rate-limit")]
            rate_limit_burst: 30,

            #[cfg(feature = "tls")]
            use_https: false,
            #[cfg(feature = "tls")]
            tls_cert_path: None,
            #[cfg(feature = "tls")]
            tls_key_path: None,
        }
    }
}

pub struct Server {
    pub options: ServerOptions,
    cors_layer: Option<CorsLayer>,
    app: Option<Router>,
}

impl Server {
    pub fn new(options: ServerOptions) -> Self {
        Server {
            options,
            cors_layer: None,
            app: None,
        }
    }

    /// adds optional `cors` to the router
    pub fn with_cors(mut self, origin_str: &str) -> Result<Self, InvalidHeaderValue> {
        let origin = origin_str.parse::<HeaderValue>()?;

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
        router: Router<State<T>>,
        state: State<T>
    ) -> Self where
        T: Send + Sync + Clone + 'static,
    {
        let mut app = router;

        if let Some(cors) = self.cors_layer.take() {
            app = app.layer(cors);
        }

        #[cfg(feature = "rate-limit")]
        {
            use tower_governor::governor::GovernorConfigBuilder;
            use tower_governor::GovernorLayer;
            use std::sync::Arc;

            let gov_config = Arc::new(GovernorConfigBuilder::default()
                .per_second(self.options.rate_limit_per_second)
                .burst_size(self.options.rate_limit_burst)
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
    pub async fn start_server(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.app.take().ok_or("Router not initialized. Did you call .with_router(router, state)?")?;

        let addr: SocketAddr = format!("{}:{}", self.options.host, self.options.port).parse()?;

        let listener = TcpListener::bind(addr).await?;

        #[cfg(feature = "tracing")]
        tracing::info!("Starting server on port {}", self.options.port);

        #[cfg(feature = "tls")]
        if self.options.use_https {
            let tls_config = match (
                &self.options.tls_cert_path,
                &self.options.tls_key_path
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

        println!("Server running on http://{}", addr);
        axum::serve(listener, app).await?;

        Ok(())
    }
}