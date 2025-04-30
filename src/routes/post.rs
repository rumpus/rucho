// Import necessary types
use axum::{
    extract::{Json, Query},        // For parsing query params and JSON bodies
    http::{HeaderMap},  // For accessing request headers and setting status codes
    response::IntoResponse,         // For building HTTP responses
};
use serde::{Deserialize, Serialize}; // For (de)serializing JSON payloads
use serde_json::json;                // Macro for easily creating JSON objects

// Import custom utilities
use crate::utils::json_response::format_json_response;  // Custom JSON response formatter
use crate::utils::error_response::format_error_response; // Custom error response formatter

// Struct for parsing optional query parameters (?pretty=true/false)
#[derive(Debug, Deserialize)]
pub struct PrettyQuery {
    pub pretty: Option<bool>, // Defaults to false if not provided
}

// Struct for deserializing any incoming JSON payload
#[derive(Debug, Deserialize, Serialize)]
pub struct Payload(serde_json::Value);

// Handler for the `/post` endpoint
// Accepts request headers, query parameters, and JSON body.
// If the body is valid JSON, returns echoed headers and body.
// If the body is invalid JSON, returns a clean error response.
pub async fn post_handler(
    headers: HeaderMap,
    Query(pretty_query): Query<PrettyQuery>,
    body: Result<Json<serde_json::Value>, axum::extract::rejection::JsonRejection>, // <-- Result!
) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);

    match body {
        Ok(Json(payload)) => {
            let response_payload = json!({
                "method": "POST",
                "headers": headers
                    .iter()
                    .map(|(k, v)| (
                        k.to_string(),
                        v.to_str().unwrap_or("<invalid utf8>").to_string()
                    ))
                    .collect::<serde_json::Value>(),
                "body": payload,
            });

            format_json_response(response_payload, pretty)
        }
        Err(_) => {
            format_error_response(axum::http::StatusCode::BAD_REQUEST, "Invalid JSON payload")
        }
    }
}

