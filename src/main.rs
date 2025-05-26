mod routes;
mod utils;

use axum::{extract::Request, Router, ServiceExt};
use tokio::{net::TcpListener, signal};
use tower_http::{
    cors::CorsLayer,
    normalize_path::NormalizePathLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing_subscriber;
use axum_server::{bind_rustls, Handle, Server};
use utils::server_config::try_load_rustls_config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let handle = Handle::new();
    let shutdown = shutdown_signal(handle.clone());

    let app = Router::new()
        .merge(routes::core_routes::router()) // Consolidated routes
        .merge(routes::healthz::router())     // Preserved
        .merge(routes::delay::router())       // Preserved
        // ----------------------------------------------------------
        // 🧱 Middleware Stack (applied top-down to all routes)
        //
        // 1. TraceLayer: Structured request/response logging using tower_http.
        //    - Customized to log at INFO level for spans, requests, and responses.
        //
        // 2. CorsLayer: Fully permissive CORS policy (for open echo testing).
        //
        // 3. NormalizePathLayer: Removes trailing slashes for route consistency.
        //
        // These are applied *after* all routes are merged, so they wrap the entire app.
        // ----------------------------------------------------------
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .layer(CorsLayer::permissive())
        .layer(NormalizePathLayer::trim_trailing_slash());

    if let Some(rustls_config) = try_load_rustls_config().await {
        tracing::info!("Starting HTTPS server on https://0.0.0.0:8443");

        bind_rustls("0.0.0.0:8443".parse().unwrap(), rustls_config)
            .handle(handle)
            .serve(app.clone().into_make_service())
            .await
            .unwrap();
    } else {
        let listener1 = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        let listener2 = TcpListener::bind("0.0.0.0:9090").await.unwrap();

        let std_listener1 = listener1.into_std().unwrap();
        let std_listener2 = listener2.into_std().unwrap();

        tracing::info!("Starting HTTP servers on :8080 and :9090");

        let serve1 = Server::from_tcp(std_listener1).serve(app.clone().into_make_service());
        let serve2 = Server::from_tcp(std_listener2).serve(app.into_make_service());

        tokio::select! {
            _ = serve1 => {},
            _ = serve2 => {},
            _ = shutdown => {
                tracing::info!("Shutting down server");
            }
        }
    }
}

async fn shutdown_signal(handle: Handle) {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("Signal received, starting graceful shutdown");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
}
