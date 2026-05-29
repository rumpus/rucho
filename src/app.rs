//! Axum application assembly.
//!
//! [`build_app`] wires every route module and the full middleware stack into a
//! single [`Router`]. It lives in the library (not the binary) so integration
//! tests can exercise the *real* app — middleware and all — via the same
//! function the server uses, rather than a hand-rolled minimal router.

use std::sync::Arc;

use axum::{extract::DefaultBodyLimit, middleware, routing::get, Router};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    normalize_path::NormalizePathLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::openapi::ApiDoc;
use crate::server::chaos_layer::chaos_middleware;
use crate::server::metrics_layer::metrics_middleware;
use crate::server::timing_layer::timing_middleware;
use crate::utils::config::ChaosConfig;
use crate::utils::metrics::Metrics;

/// Builds the Axum application with all routes and middleware.
///
/// If `metrics` is `Some`, enables the `/metrics` endpoint and metrics-collection
/// middleware. If `compression_enabled` is true, enables gzip/brotli response
/// compression. If chaos mode is enabled, adds chaos middleware for resilience
/// testing. `max_body_size_bytes` caps request body size via `DefaultBodyLimit`;
/// requests with larger bodies receive 413 Payload Too Large.
pub fn build_app(
    metrics: Option<Arc<Metrics>>,
    compression_enabled: bool,
    chaos: Arc<ChaosConfig>,
    max_body_size_bytes: usize,
) -> Router {
    let mut app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(crate::routes::core_routes::router())
        .merge(crate::routes::healthz::router())
        .merge(crate::routes::delay::router())
        .merge(crate::routes::redirect::router())
        .merge(crate::routes::cookies::router())
        .merge(crate::routes::base64::router())
        .merge(crate::routes::bytes::router())
        .merge(crate::routes::drip::router())
        .merge(crate::routes::response_headers::router())
        .merge(crate::routes::content_types::router())
        .merge(crate::routes::image::router())
        .merge(crate::routes::range::router())
        .layer(DefaultBodyLimit::max(max_body_size_bytes));

    // Add metrics endpoint and middleware if enabled
    if let Some(metrics) = metrics {
        app = app
            .route(
                "/metrics",
                get(crate::routes::metrics::get_metrics).with_state(metrics.clone()),
            )
            .layer(middleware::from_fn(move |req, next| {
                let metrics = metrics.clone();
                async move { metrics_middleware(req, next, metrics).await }
            }));
    }

    // Middleware order (innermost to outermost):
    // routes → chaos → timing → trace → compression → cors → normalize-path
    // Chaos sits inside timing so duration_ms honestly reflects chaos delays.
    let app = if chaos.is_enabled() {
        app.layer(middleware::from_fn(move |req, next| {
            let chaos = chaos.clone();
            async move { chaos_middleware(req, next, chaos).await }
        }))
    } else {
        app
    };

    let app = app.layer(middleware::from_fn(timing_middleware)).layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO)),
    );

    // Conditionally add compression layer
    let app = if compression_enabled {
        tracing::info!("Response compression enabled (gzip, brotli)");
        app.layer(CompressionLayer::new())
    } else {
        app
    };

    app.layer(CorsLayer::permissive())
        .layer(NormalizePathLayer::trim_trailing_slash())
}
