// anything.rs
use axum::{routing::any, Router, body::{Body, to_bytes}, extract::{OriginalUri, Query}, http::{HeaderMap, Method}, response::IntoResponse};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use crate::utils::json_response::format_json_response;

#[derive(Debug, Deserialize)]
pub struct PrettyQuery {
    pretty: Option<bool>,
}

pub fn router() -> Router {
    Router::new()
        .route("/anything", any(anything_handler))
        .route("/anything/*path", any(anything_handler))
}

async fn anything_handler(method: Method, OriginalUri(uri): OriginalUri, headers: HeaderMap, Query(query): Query<PrettyQuery>, body: Body) -> impl IntoResponse {
    let body_bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return format_json_response(json!({"error": "Failed to read body"}), query.pretty.unwrap_or(false)),
    };

    let headers_json: Value = headers.iter().map(|(k, v)| (
        k.to_string(),
        Value::String(v.to_str().unwrap_or("<invalid utf8>").to_string())
    )).collect::<Map<_, _>>().into();

    let resp = json!({
        "method": method.to_string(),
        "path": uri.path(),
        "query": uri.query().unwrap_or(""),
        "headers": headers_json,
        "body": String::from_utf8_lossy(&body_bytes),
    });

    format_json_response(resp, query.pretty.unwrap_or(false))
}
