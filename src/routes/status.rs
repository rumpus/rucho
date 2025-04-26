use axum::{
    extract::Path,
    http::StatusCode,
    response::Response,
};

pub async fn status_handler(Path(code): Path<u16>) -> Response {
    let status = StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST);

    Response::builder()
        .status(status)
        .body(axum::body::Body::empty())
        .unwrap()
}
