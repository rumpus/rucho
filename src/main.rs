//! Rucho server entry point.
//!
//! This is the main entry point for the Rucho application. It handles CLI argument
//! parsing and dispatches to the appropriate command handlers.

use axum::{middleware, routing::get, Router};
use clap::Parser;
use std::str::FromStr;
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    normalize_path::NormalizePathLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use rucho::cli::{
    commands::{
        handle_start_command, handle_status_command, handle_stop_command, handle_version_command,
    },
    Args, CliCommand,
};
use rucho::routes::core_routes::EndpointInfo;
use rucho::server::metrics_layer::metrics_middleware;
use rucho::server::timing_layer::timing_middleware;
use rucho::utils::config::Config;
use rucho::utils::metrics::Metrics;

#[derive(OpenApi)]
#[openapi(
    paths(
        rucho::routes::core_routes::root_handler,
        rucho::routes::core_routes::get_handler,
        rucho::routes::core_routes::head_handler,
        rucho::routes::core_routes::post_handler,
        rucho::routes::core_routes::put_handler,
        rucho::routes::core_routes::patch_handler,
        rucho::routes::core_routes::delete_handler,
        rucho::routes::core_routes::options_handler,
        rucho::routes::core_routes::status_handler,
        rucho::routes::core_routes::anything_handler,
        rucho::routes::core_routes::anything_path_handler,
        rucho::routes::core_routes::endpoints_handler,
        rucho::routes::delay::delay_handler,
        rucho::routes::healthz::healthz_handler,
    ),
    components(
        schemas(EndpointInfo, rucho::routes::core_routes::Payload)
    ),
    tags(
        (name = "Rucho", description = "Rucho API")
    )
)]
/// Represents the OpenAPI documentation structure.
///
/// This struct is used by `utoipa` to generate the OpenAPI specification
/// for the Rucho API. It aggregates all the paths and components
/// (schemas, responses, etc.) that are part of the API.
struct ApiDoc;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::load();

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    // Initialize tracing with configured log level
    let log_level = Level::from_str(&config.log_level.to_uppercase()).unwrap_or_else(|_| {
        eprintln!(
            "Warning: Invalid log level '{}' in config, defaulting to INFO.",
            config.log_level
        );
        Level::INFO
    });
    tracing_subscriber::fmt().with_max_level(log_level).init();

    // Dispatch command
    match args.command {
        CliCommand::Start {} => {
            if handle_start_command() {
                // Create metrics store if enabled
                let metrics = if config.metrics_enabled {
                    tracing::info!("Metrics endpoint enabled at /metrics");
                    Some(Arc::new(Metrics::new()))
                } else {
                    None
                };

                let app = build_app(metrics);
                rucho::server::run_server(&config, app).await;
            }
        }
        CliCommand::Stop {} => handle_stop_command(),
        CliCommand::Status {} => handle_status_command(),
        CliCommand::Version {} => handle_version_command(),
    }
}

/// Builds the Axum application with all routes and middleware.
///
/// If metrics is Some, enables the /metrics endpoint and metrics collection middleware.
fn build_app(metrics: Option<Arc<Metrics>>) -> Router {
    let mut app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(rucho::routes::core_routes::router())
        .merge(rucho::routes::healthz::router())
        .merge(rucho::routes::delay::router());

    // Add metrics endpoint and middleware if enabled
    if let Some(metrics) = metrics {
        app = app
            .route(
                "/metrics",
                get(rucho::routes::metrics::get_metrics).with_state(metrics.clone()),
            )
            .layer(middleware::from_fn(move |req, next| {
                let metrics = metrics.clone();
                async move { metrics_middleware(req, next, metrics).await }
            }));
    }

    app.layer(middleware::from_fn(timing_middleware))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CorsLayer::permissive())
        .layer(NormalizePathLayer::trim_trailing_slash())
}
