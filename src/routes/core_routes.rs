use axum::{
    routing::{get, post, put, patch, delete, options, head, any}, 
    Router,
    extract::{Json, Query, Path, OriginalUri}, 
    http::{HeaderMap, Method, StatusCode}, 
    response::{IntoResponse, Response},
    body::{Body, to_bytes},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::utils::{
    json_response::format_json_response,
    error_response::format_error_response,
    request_models::PrettyQuery,
};

// This Payload struct is used by post, put, patch, delete handlers. Define it once.
#[derive(Debug, Deserialize, Serialize)]
struct Payload(serde_json::Value);

pub fn router() -> Router {
    Router::new()
        // Routes from get.rs
        .route("/", get(root_handler)) 
        .route("/get", get(get_handler))
        .route("/get", head(head_handler))
        // Routes from post.rs
        .route("/post", post(post_handler))
        // Routes from put.rs
        .route("/put", put(put_handler))
        // Routes from patch.rs
        .route("/patch", patch(patch_handler))
        // Routes from delete.rs
        .route("/delete", delete(delete_handler))
        // Routes from options.rs
        .route("/options", options(options_handler))
        // Route from status.rs
        .route("/status/:code", any(status_handler))
        // Routes from anything.rs
        .route("/anything", any(anything_handler))
        .route("/anything/*path", any(anything_handler))
}

// From get.rs
async fn root_handler() -> &'static str {
    "Welcome to Echo Server!
"
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

// From status.rs
async fn status_handler(Path(code): Path<u16>, _method: Method) -> Response {
    StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST).into_response()
}

// From anything.rs
async fn anything_handler(
    method: Method, 
    OriginalUri(uri): OriginalUri, 
    headers: HeaderMap, 
    Query(query): Query<PrettyQuery>, // Uses crate::utils::request_models::PrettyQuery
    body: Body
) -> impl IntoResponse {
    let pretty = query.pretty.unwrap_or(false); // Adjusted to use the imported PrettyQuery
    let body_bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        // format_json_response is already in scope
        Err(_) => return format_json_response(json!({"error": "Failed to read body"}), pretty), 
    };

    // serde_json::Value is Value, Map is Map, json! macro is available
    let headers_json: serde_json::Value = headers.iter().map(|(k, v)| (
        k.to_string(),
        serde_json::Value::String(v.to_str().unwrap_or("<invalid utf8>").to_string())
    )).collect::<serde_json::Map<_, _>>().into();

    let resp = json!({
        "method": method.to_string(),
        "path": uri.path(),
        "query": uri.query().unwrap_or(""),
        "headers": headers_json,
        "body": String::from_utf8_lossy(&body_bytes), // This is correct
    });

    format_json_response(resp, pretty)
}

// From post.rs
async fn post_handler(
    headers: HeaderMap, 
    Query(pretty_query): Query<PrettyQuery>, 
    body: Result<Json<serde_json::Value>, axum::extract::rejection::JsonRejection> 
) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);
    match body {
        Ok(Json(payload_value)) => { 
            let response_payload = json!({
                "method": "POST",
                "headers": headers.iter().map(|(k, v)| (
                    k.to_string(),
                    v.to_str().unwrap_or("<invalid utf8>").to_string()
                )).collect::<serde_json::Value>(),
                "body": payload_value, 
            });
            format_json_response(response_payload, pretty)
        }
        Err(_) => format_error_response(axum::http::StatusCode::BAD_REQUEST, "Invalid JSON payload")
    }
}

// From put.rs
async fn put_handler(
    headers: HeaderMap, 
    Query(pretty_query): Query<PrettyQuery>, 
    body: Result<Json<Payload>, axum::extract::rejection::JsonRejection>
) -> impl IntoResponse {
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

// From patch.rs
async fn patch_handler(
    headers: HeaderMap, 
    Query(pretty_query): Query<PrettyQuery>, 
    body: Result<Json<Payload>, axum::extract::rejection::JsonRejection>
) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);
    match body {
        Ok(Json(Payload(body_json))) => {
            let payload = json!({
                "method": "PATCH",
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

// From delete.rs
async fn delete_handler(
    headers: HeaderMap, 
    Query(pretty_query): Query<PrettyQuery>, 
    body: Result<Json<Payload>, axum::extract::rejection::JsonRejection> 
) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);
    match body {
        Ok(Json(Payload(body_json))) => {
            let payload = json!({
                "method": "DELETE",
                "headers": headers.iter().map(|(k, v)| (
                    k.to_string(),
                    v.to_str().unwrap_or("<invalid utf8>").to_string()
                )).collect::<serde_json::Value>(),
                "body": body_json, 
            });
            format_json_response(payload, pretty)
        }
        Err(_) => { 
             let payload = json!({
                "method": "DELETE",
                "headers": headers.iter().map(|(k, v)| (
                    k.to_string(),
                    v.to_str().unwrap_or("<invalid utf8>").to_string()
                )).collect::<serde_json::Value>(),
                "body": serde_json::Value::Null, 
            });
            format_json_response(payload, pretty)
        }
    }
}

// From options.rs
async fn options_handler() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::NO_CONTENT) 
        .header(axum::http::header::ALLOW, "GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD") 
        .body(axum::body::Body::empty())
        .unwrap()
}
