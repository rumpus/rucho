// post.rs
use axum::{routing::post, Router, extract::{Json, Query}, http::HeaderMap, response::IntoResponse};
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
    Router::new().route("/post", post(post_handler))
}

async fn post_handler(headers: HeaderMap, Query(pretty_query): Query<PrettyQuery>, body: Result<Json<serde_json::Value>, axum::extract::rejection::JsonRejection>) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);
    match body {
        Ok(Json(payload)) => {
            let response_payload = json!({
                "method": "POST",
                "headers": headers.iter().map(|(k, v)| (
                    k.to_string(),
                    v.to_str().unwrap_or("<invalid utf8>").to_string()
                )).collect::<serde_json::Value>(),
                "body": payload,
            });
            format_json_response(response_payload, pretty)
        }
        Err(_) => format_error_response(axum::http::StatusCode::BAD_REQUEST, "Invalid JSON payload")
    }
}