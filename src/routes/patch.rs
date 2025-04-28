// Import necessary types
use axum::{
    extract::{Query},
    http::HeaderMap,
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

// Handler for the `/patch` endpoint
// Accepts request headers and body, and returns them in a JSON response
pub async fn patch_handler(
    headers: HeaderMap,
    Query(pretty_query): Query<PrettyQuery>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);

    let body_text = String::from_utf8_lossy(&body);

    let payload = json!({
        "method": "PATCH",
        "headers": headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("<invalid utf8>").to_string()))
            .collect::<serde_json::Value>(),
        "body": body_text,
    });

    format_json_response(payload, pretty)
}
