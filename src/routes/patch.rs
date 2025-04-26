use axum::extract::Json;
use serde_json::Value;

pub async fn patch_handler(Json(payload): Json<Value>) -> Json<Value> {
    Json(payload)
}
