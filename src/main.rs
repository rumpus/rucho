// Main entry point for the Echo Server

mod routes;
mod utils;

use axum::{
    routing::{get, post, put, patch, delete, options},
    Router,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use tokio::net::TcpListener;
use tokio::signal;

//use axum_server::Server;
//use rustls::Connection::Server;
//use rustls::Side::Server;
//use rustls::quic::Connection::Server;

use axum_server::{Handle, Server, bind_rustls};
use utils::server_config::try_load_rustls_config;

// Route modules
use routes::{
    get as get_routes,
    post as post_routes,
    put as put_routes,
    patch as patch_routes,
    delete as delete_routes,
    options as options_routes,
    status as status_routes,
    anything::anything_handler,
    healthz::healthz_handler,
    delay::delay_handler,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Shared shutdown handle
    let handle = Handle::new();
    let shutdown = shutdown_signal(handle.clone());

    let app = Router::new()
        .route("/", get(get_routes::root))
        .route("/get", get(get_routes::get_handler))
        .route("/get", axum::routing::head(get_routes::head_handler))
        .route("/post", post(post_routes::post_handler))
        .route("/put", put(put_routes::put_handler))
        .route("/patch", patch(patch_routes::patch_handler))
        .route("/delete", delete(delete_routes::delete_handler))
        .route("/options", options(options_routes::options_handler))
        .route("/status/:code", get(status_routes::status_handler))
        .route("/anything", axum::routing::any(anything_handler))
        .route("/healthz", get(healthz_handler))
        .route("/delay/:n", get(delay_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new()
                        .level(tracing::Level::INFO),
                )
                .on_request(
                    tower_http::trace::DefaultOnRequest::new()
                        .level(tracing::Level::INFO),
                )
                .on_response(
                    tower_http::trace::DefaultOnResponse::new()
                        .level(tracing::Level::INFO),
                ),
        );

    if let Some(rustls_config) = try_load_rustls_config().await {
        tracing::info!("Starting HTTPS server on https://0.0.0.0:8443");

        bind_rustls("0.0.0.0:8443".parse().unwrap(), rustls_config)
            .handle(handle)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        let std_listener = listener.into_std().unwrap(); // Convert to std::net::TcpListener
        tracing::info!("Starting HTTP server on http://{}", std_listener.local_addr().unwrap());

        Server::from_tcp(std_listener)
            .serve(app.into_make_service())
            //.with_graceful_shutdown(shutdown)
            .await
            .unwrap();
    }
}

async fn shutdown_signal(handle: Handle) {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("Signal received, starting graceful shutdown");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
}
