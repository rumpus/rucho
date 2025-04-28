// Import necessary types
use axum::{
    extract::{Query},
    http::HeaderMap,                     // Represents the incoming HTTP request headers
    response::{IntoResponse, Response},  // For building responses
};
use serde::Deserialize;                  // For parsing query parameters
use serde_json::json;                     // Macro for easily creating JSON objects
use crate::utils::json_response::format_json_response;  // Custom helper to format JSON responses

// Struct for parsing optional query parameters
#[derive(Debug, Deserialize)]
pub struct PrettyQuery {
    pub pretty: Option<bool>,             // ?pretty=true/false
}

// Handler for the root endpoint `/`
// Just returns a static welcome message
pub async fn root() -> &'static str {
    "Welcome to Echo Server!\n"
}

// Handler for the `/get` endpoint
// Returns a JSON response with the request method and headers
// Supports optional `?pretty=true` query parameter for pretty-printed output
pub async fn get_handler(
    headers: HeaderMap,                  // Incoming request headers
    Query(pretty_query): Query<PrettyQuery>, // Extracted query parameters
) -> Response {
    let pretty = pretty_query.pretty.unwrap_or(false); // Default to compact unless ?pretty=true

    let payload = json!({
        "method": "GET",                                                // Indicate the HTTP method
        "headers": headers
            .iter()
            .map(|(k, v)| (
                k.to_string(),                                          // Convert header key to String
                v.to_str().unwrap_or("<invalid utf8>").to_string()      // Safely convert value or show placeholder
            ))
            .collect::<serde_json::Value>(),                            // Collect into a JSON object
    });

    // Use the utility function to format the JSON into an Axum Response
    format_json_response(payload, pretty)
}

// Handler for HEAD requests to `/get`
// Returns only status code and headers (no body)
pub async fn head_handler() -> impl IntoResponse {
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .body(axum::body::Body::empty())
        .unwrap()
}
