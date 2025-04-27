// Import necessary types
use http::HeaderMap;                                    // Represents the incoming HTTP request headers
use serde_json::json;                                   // Macro for easily creating JSON objects
use crate::utils::json_response::format_json_response;  // Custom helper to format JSON responses

// Handler for the root endpoint `/`
// Just returns a static welcome message
pub async fn root() -> &'static str {
    "Welcome to Echo Server!\n"
}

// Handler for the `/get` endpoint
// Returns a JSON response with the request method and headers
pub async fn get_handler(headers: HeaderMap) -> axum::response::Response {
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
    format_json_response(&payload)
}