use http::HeaderMap;
use serde_json::json;
use crate::utils::json_response::format_json_response;

pub async fn root() -> &'static str {
    "Welcome to Echo Server!\n"
}

pub async fn get_handler(headers: HeaderMap) -> axum::response::Response {
    let payload = json!({
        "method": "GET",
        "headers": headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("<invalid utf8>").to_string()))
            .collect::<serde_json::Value>(),
    });

    format_json_response(&payload)
}