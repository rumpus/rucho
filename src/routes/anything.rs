// Handler for the /anything endpoint
// Catches any HTTP method (GET, POST, PUT, PATCH, DELETE, etc.)
// and echoes back request method, path, query params, headers, and body.

use axum::{
    body::Bytes,
    extract::{OriginalUri, Query},
    http::{Method, HeaderMap},
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use crate::utils::json_response::format_json_response;

// Struct for parsing optional query parameters
#[derive(Debug, Deserialize)]
pub struct PrettyQuery {
    pub pretty: Option<bool>, // ?pretty=true/false
}

pub async fn anything_handler(
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    Query(pretty_query): Query<PrettyQuery>,
    body: Bytes,
) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);

    let body_text = String::from_utf8_lossy(&body);

    let payload = json!({
        "method": method.to_string(),
        "path": uri.path(),
        "query": uri.query().unwrap_or("").to_string(),
        "headers": headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("<invalid utf8>").to_string()))
            .collect::<serde_json::Value>(),
        "body": body_text,
    });

    format_json_response(payload, pretty)
}
