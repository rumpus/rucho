// Main entry point for the Echo Server

mod routes;
mod utils;

use axum::{
    routing::{get, post, put, patch, delete, options, any},
    Router,
};

use tower_http::trace::TraceLayer;
use tracing_subscriber;
use tokio::net::TcpListener;
use tokio::signal;
//use std::sync::Arc;
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
    status::status_handler,
    //status as status_routes,
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
        .route("/status/:code", any(status_handler))
        .route("/anything", any(anything_handler))
        .route("/healthz", get(healthz_handler))
        .route("/delay/:n", any(delay_handler))
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

    let app1 = app.clone();
    let app2 = app.clone();    

    if let Some(rustls_config) = try_load_rustls_config().await {
        tracing::info!("Starting HTTPS server on https://0.0.0.0:8443");

        bind_rustls("0.0.0.0:8443".parse().unwrap(), rustls_config)
            .handle(handle)
            .serve(app.clone().into_make_service()) //.serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        let listener1 = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        let listener2 = TcpListener::bind("0.0.0.0:9090").await.unwrap();

        let std_listener1 = listener1.into_std().unwrap(); // Convert to std::net::TcpListener
        let std_listener2 = listener2.into_std().unwrap(); // Convert to std::net::TcpListener

        tracing::info!("Starting HTTP server on http://0.0.0.0:8080 and http://0.0.0.0:9090"); // tracing::info!("Starting HTTP server on http://0.0.0.0:8080 and http://0.0.0.0:9090");

        let serve1 = Server::from_tcp(std_listener1).serve(app1.clone().into_make_service());
        let serve2 = Server::from_tcp(std_listener2).serve(app2.clone().into_make_service());

        //let serve1 = Server::from_tcp(std_listener1).serve(app1.into_make_service());
        //let serve2 = Server::from_tcp(std_listener2).serve(app2.into_make_service());

        tokio::select! {
            _ = serve1 => {},
            _ = serve2 => {},
            _ = shutdown => {
                tracing::info!("Shutting down server");
            }
        }

        // Server::from_tcp(std_listener)
        //     .serve(app.into_make_service())
        //     //.with_graceful_shutdown(shutdown)
        //     .await
        //     .unwrap();
    }
}

async fn shutdown_signal(handle: Handle) {
    signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("Signal received, starting graceful shutdown");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(5)));
}
