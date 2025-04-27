// Import necessary types
use axum::{
    body::Body,           // Represents the HTTP response body
    http::Response,       // Represents the full HTTP Response
};
use serde_json::Value;     // Represents arbitrary JSON values

/// Takes a JSON Value and returns a properly formatted HTTP Response
pub fn format_json_response(payload: &Value) -> Response<Body> {
    // Serialize the JSON Value into a String
    let mut body = serde_json::to_string(payload).unwrap();
    
    // Append a newline at the end for nicer formatting (optional, but nice for curl/readability)
    body.push('\n');

    // Build and return the HTTP Response
    Response::builder()
        .status(200)                             // Always returns HTTP 200 OK
        .header("Content-Type", "application/json") // Set the Content-Type header manually
        .body(Body::from(body))                  // Set the response body
        .unwrap()                                // Safe unwrap: static, known-good response building
}