// Main entry point for the Echo Server

// Declare the `routes` and `utils` modules (located in src/routes and src/utils folders)
mod routes;
mod utils;

// Bring in necessary items from external crates
use axum::{
    routing::{get, post, put, patch, delete, options},  // HTTP method handlers
    Router, serve                                       // Main Router and function to server the app
};
use tower_http::trace::TraceLayer;                      // For automatic HTTP request/response tracing
use tracing_subscriber;                                 // Subscriber for structured logging (tracing)
use tokio::net::TcpListener;                            // Async TCP listener

// Bring in each grp of route handlers, namespaced by METHOD
use routes::{
    get as get_routes,
    post as post_routes,
    put as put_routes,
    patch as patch_routes,
    delete as delete_routes,
    options as options_routes,
    status as status_routes, // Status route for dynamic status code responses
};


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();        // initilize the tracing subscriber for structured logging output

    let app = Router::new()     // Build app router by chaining route definitions
    .route("/", get(get_routes::root))
    .route("/get", get(get_routes::get_handler))
    .route("/post", post(post_routes::post_handler))
    .route("/put", put(put_routes::put_handler))
    .route("/patch", patch(patch_routes::patch_handler))
    .route("/delete", delete(delete_routes::delete_handler))
    .route("/options", options(options_routes::options_handler))
    .route("/status/:code", get(status_routes::status_handler))

    // Add a middleware layer to trace HTTP req and resps
    .layer(
        TraceLayer::new_for_http()
            .make_span_with(
                tower_http::trace::DefaultMakeSpan::new()
                    .level(tracing::Level::INFO),       // Set logging level
            )
            .on_request(
                tower_http::trace::DefaultOnRequest::new()
                    .level(tracing::Level::INFO),       // Set incoming req
            )
            .on_response(
                tower_http::trace::DefaultOnResponse::new()
                    .level(tracing::Level::INFO),       // Set outgoing resp
            ),
    );

    // Bind a TCP listener to 0.0.0.0:8080 (listen on all interfaces, port 8080)
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    // Log the address that the server is listening on
    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    // Serve the app using the listener
    serve(listener, app).await.unwrap();
}
