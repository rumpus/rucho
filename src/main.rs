mod routes;
mod utils;

use axum::{
    routing::{get, post, put, patch, delete, options},
    Router, serve
};
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use tokio::net::TcpListener;
use routes::{
    get as get_routes,
    post as post_routes,
    put as put_routes,
    patch as patch_routes,
    delete as delete_routes,
    options as options_routes,
    status as status_routes, // <-- NEW
};


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
    .route("/", get(get_routes::root))
    .route("/get", get(get_routes::get_handler))
    .route("/post", post(post_routes::post_handler))
    .route("/put", put(put_routes::put_handler))
    .route("/patch", patch(patch_routes::patch_handler))
    .route("/delete", delete(delete_routes::delete_handler))
    .route("/options", options(options_routes::options_handler))
    .route("/status/:code", get(status_routes::status_handler))
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

    
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    serve(listener, app).await.unwrap();
}
