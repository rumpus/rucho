// Import necessary types from Axum and Serde
use axum::{http::StatusCode, response::Response};
use serde_json::{json, Value};

/// Formats a `serde_json::Value` into an Axum `Response`.
///
/// This function serializes the given JSON `Value` into a pretty-printed string.
/// The response will have an HTTP 200 OK status and a "Content-Type: application/json" header.
///
/// # Arguments
///
/// * `data`: A `serde_json::Value` to be serialized and sent in the response body.
///
/// # Returns
///
/// An Axum `Response` object. Returns a 500 error response if serialization fails.
pub fn format_json_response(data: Value) -> Response {
    format_json_response_with_timing(data, None)
}

/// Formats a `serde_json::Value` into an Axum `Response` with optional timing information.
///
/// This function serializes the given JSON `Value` into a pretty-printed string.
/// If `duration_ms` is provided, a `timing` object is added to the response.
/// The response will have an HTTP 200 OK status and a "Content-Type: application/json" header.
///
/// # Arguments
///
/// * `data`: A `serde_json::Value` to be serialized and sent in the response body.
/// * `duration_ms`: Optional request duration in milliseconds.
///
/// # Returns
///
/// An Axum `Response` object. Returns a 500 error response if serialization fails.
pub fn format_json_response_with_timing(mut data: Value, duration_ms: Option<f64>) -> Response {
    // If timing is provided and data is an object, inject it
    if let Some(ms) = duration_ms {
        if let Some(obj) = data.as_object_mut() {
            obj.insert("timing".to_string(), json!({ "duration_ms": ms }));
        }
    }

    let body = serde_json::to_string_pretty(&data);

    match body {
        Ok(json_string) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(json_string))
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(axum::body::Body::from(
                        r#"{"error":"Failed to build response"}"#,
                    ))
                    .expect("fallback response should always build")
            }),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(
                r#"{"error":"Failed to serialize response"}"#,
            ))
            .expect("fallback response should always build"),
    }
}
