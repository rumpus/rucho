use axum::{
    body::Body,
    http::Response,
};
use serde_json::Value;

/// Takes a JSON Value and returns a properly formatted Response
pub fn format_json_response(payload: &Value) -> Response<Body> {
    let mut body = serde_json::to_string(payload).unwrap();
    body.push('\n');

    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}