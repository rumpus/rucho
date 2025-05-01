// put.rs
use axum::{routing::put, Router, extract::{Json, Query}, http::HeaderMap, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::utils::{json_response::format_json_response, error_response::format_error_response};

#[derive(Debug, Deserialize)]
pub struct PrettyQuery {
    pub pretty: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Payload(serde_json::Value);

pub fn router() -> Router {
    Router::new().route("/put", put(put_handler))
}

async fn put_handler(headers: HeaderMap, Query(pretty_query): Query<PrettyQuery>, body: Result<Json<Payload>, axum::extract::rejection::JsonRejection>) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);
    match body {
        Ok(Json(Payload(body_json))) => {
            let payload = json!({
                "method": "PUT",
                "headers": headers.iter().map(|(k, v)| (
                    k.to_string(),
                    v.to_str().unwrap_or("<invalid utf8>").to_string()
                )).collect::<serde_json::Value>(),
                "body": body_json,
            });
            format_json_response(payload, pretty)
        }
        Err(_) => format_error_response(axum::http::StatusCode::BAD_REQUEST, "Invalid JSON payload")
    }
}