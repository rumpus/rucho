// Import necessary types
use axum::{
    extract::{Json, Query},
    http::HeaderMap,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::utils::json_response::format_json_response;
use crate::utils::error_response::format_error_response; // <-- NEW: error formatting helper

// Struct for parsing optional query parameters
#[derive(Debug, Deserialize)]
pub struct PrettyQuery {
    pub pretty: Option<bool>, // ?pretty=true/false
}

// Struct for deserializing any incoming JSON payload
#[derive(Debug, Deserialize, Serialize)]
pub struct Payload(serde_json::Value);

// Handler for the `/delete` endpoint
// Accepts request headers, query params, and parsed JSON body
pub async fn delete_handler(
    headers: HeaderMap,
    Query(pretty_query): Query<PrettyQuery>,
    body: Result<Json<Payload>, axum::extract::rejection::JsonRejection>, // <-- Result for validation
) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);

    match body {
        Ok(Json(Payload(body_json))) => {
            let payload = json!({
                "method": "DELETE",
                "headers": headers
                    .iter()
                    .map(|(k, v)| (
                        k.to_string(),
                        v.to_str().unwrap_or("<invalid utf8>").to_string()
                    ))
                    .collect::<serde_json::Value>(),
                "body": body_json,
            });

            format_json_response(payload, pretty)
        }
        Err(_) => {
            format_error_response(axum::http::StatusCode::BAD_REQUEST, "Invalid JSON payload")
        }
    }
}
