use axum::{
    response::IntoResponse,
    http::StatusCode,
};

pub async fn healthz_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
