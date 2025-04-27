use axum::extract::Json;
use serde_json::Value;
use crate::utils::json_response::format_json_response;

pub async fn delete_handler(Json(payload): Json<Value>) -> axum::response::Response {
    format_json_response(&payload)
}