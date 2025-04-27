// Import necessary types
use axum::extract::Json;                                // Extractor for automatically parsing JSON bodies
use serde_json::Value;                                  // Represents arbitrary JSON values
use crate::utils::json_response::format_json_response;  // Custom utility to format JSON responses

// Handler for the `/delete` endpoint
// Accepts a JSON payload and echoes it back in the response
pub async fn delete_handler(Json(payload): Json<Value>) -> axum::response::Response {
    // Use the utility function to format and return the JSON payload
    format_json_response(&payload)
}