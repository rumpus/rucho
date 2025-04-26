pub async fn root() -> &'static str {
    "Welcome to Echo Server!"
}

pub async fn get_handler(headers: axum::http::HeaderMap) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "method": "GET",
        "headers": headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("<invalid utf8>").to_string()))
            .collect::<serde_json::Value>(),
    }))
}

use axum::extract::Json;
use serde_json::Value;

pub async fn post_handler(Json(payload): Json<Value>) -> Json<Value> {
    Json(payload)
}
