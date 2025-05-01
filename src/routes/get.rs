use axum::{routing::{get, head}, Router, extract::Query, http::HeaderMap, response::{IntoResponse, Response}};
use serde::Deserialize;
use serde_json::json;
use crate::utils::json_response::format_json_response;

#[derive(Debug, Deserialize)]
pub struct PrettyQuery {
    pub pretty: Option<bool>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/get", get(get_handler))
        .route("/get", head(head_handler))
}

async fn root() -> &'static str {
    "Welcome to Echo Server!\n"
}

async fn get_handler(
    headers: HeaderMap,
    Query(pretty_query): Query<PrettyQuery>,
) -> Response {
    let pretty = pretty_query.pretty.unwrap_or(false);

    let payload = json!({
        "method": "GET",
        "headers": headers.iter().map(|(k, v)| (
            k.to_string(),
            v.to_str().unwrap_or("<invalid utf8>").to_string()
        )).collect::<serde_json::Value>(),
    });

    format_json_response(payload, pretty)
}

async fn head_handler() -> impl IntoResponse {
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .body(axum::body::Body::empty())
        .unwrap()
}
