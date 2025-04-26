mod handlers;

use axum::{routing::{get, post}, Router, serve};
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use tokio::net::TcpListener;
use routes::{get as get_routes, post as post_routes}; // <-- NEW


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
    .route("/", get(get_routes::root))
    .route("/get", get(get_routes::get_handler))
    .route("/post", post(post_routes::post_handler))
    .layer(TraceLayer::new_for_http());



    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    serve(listener, app).await.unwrap();
}
