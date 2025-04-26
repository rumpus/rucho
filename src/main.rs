mod handlers;

use axum::{routing::{get, post}, Router, serve};
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
    .route("/", get(handlers::echo::root))
    .route("/get", get(handlers::echo::get_handler))
    .route("/post", post(handlers::echo::post_handler)) // <--- add this line
    .layer(TraceLayer::new_for_http());


    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    serve(listener, app).await.unwrap();
}
