use axum::{
    extract::Json,
    body::Body,
    http::Response,
};
use serde_json::Value;

pub async fn put_handler(Json(payload): Json<Value>) -> Response<Body> {
    let mut body = serde_json::to_string(&payload).unwrap();
    body.push('\n');

    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}
