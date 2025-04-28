// Main entry point for the Echo Server

// Declare the `routes` and `utils` modules (located in src/routes and src/utils folders)
mod routes;
mod utils;

// Bring in necessary items from external crates
use axum::{
    routing::{get, post, put, patch, delete, options},  // HTTP method handlers
    Router, serve                                       // Main Router and serve function
};
use tower_http::trace::TraceLayer;                      // Middleware for automatic HTTP request/response tracing
use tracing_subscriber;                                 // Structured logging with tracing
use tokio::net::TcpListener;                            // Async TCP listener

// Bring in grouped route handlers, namespaced by HTTP METHOD
use routes::{
    get as get_routes,
    post as post_routes,
    put as put_routes,
    patch as patch_routes,
    delete as delete_routes,
    options as options_routes,
    status as status_routes,   // Handles dynamic status code responses
    anything::anything_handler, // Handles /anything route for any method
};

#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber for structured logs
    tracing_subscriber::fmt::init();

    // Build the application router
    let app = Router::new()
        .route("/", get(get_routes::root))
        .route("/get", get(get_routes::get_handler))
        .route("/post", post(post_routes::post_handler))
        .route("/put", put(put_routes::put_handler))
        .route("/patch", patch(patch_routes::patch_handler))
        .route("/delete", delete(delete_routes::delete_handler))
        .route("/options", options(options_routes::options_handler))
        .route("/status/:code", get(status_routes::status_handler))
        .route("/anything", axum::routing::any(anything_handler))
        // Add a middleware layer to trace HTTP requests and responses
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new()
                        .level(tracing::Level::INFO),  // Set log level for span creation
                )
                .on_request(
                    tower_http::trace::DefaultOnRequest::new()
                        .level(tracing::Level::INFO),  // Log incoming requests
                )
                .on_response(
                    tower_http::trace::DefaultOnResponse::new()
                        .level(tracing::Level::INFO),  // Log outgoing responses
                ),
        );

    // Bind a TCP listener to 0.0.0.0:8080 (listen on all interfaces, port 8080)
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    // Log the address that the server is listening on
    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    // Serve the app using the listener
    serve(listener, app).await.unwrap();
}
