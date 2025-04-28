// Handler for the /anything endpoint
// Catches any HTTP method (GET, POST, PUT, PATCH, DELETE, etc.)
// and echoes back request method, path, query params, headers, and body.

use axum::{
    body::Bytes,
    extract::OriginalUri,
    http::{Method, HeaderMap},
    response::Json,
};
use serde_json::json;
use std::collections::HashMap;

pub async fn anything_handler(
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> Json<serde_json::Value> {
    let body_text = String::from_utf8_lossy(&body);

    Json(json!({
        "method": method.to_string(),
        "path": uri.path(),
        "query": uri.query().unwrap_or("").to_string(),
        "headers": headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("<invalid utf8>").to_string()))
            .collect::<HashMap<_, _>>(),
        "body": body_text,
    }))
}
