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

#[derive(Serialize, Debug, Clone, Copy)] // Added Serialize
pub struct EndpointInfo {
    path: &'static str,
    method: &'static str,
    description: &'static str,
}

static API_ENDPOINTS: &[EndpointInfo] = &[
    // Routes from former get.rs
    EndpointInfo { path: "/", method: "GET", description: "Root welcome message." },
    EndpointInfo { path: "/get", method: "GET", description: "Echoes request details for GET." },
    EndpointInfo { path: "/get", method: "HEAD", description: "Responds with headers for GET query." },
    // Routes from former post.rs
    EndpointInfo { path: "/post", method: "POST", description: "Echoes request details for POST, expects JSON body." },
    // Routes from former put.rs
    EndpointInfo { path: "/put", method: "PUT", description: "Echoes request details for PUT, expects JSON body." },
    // Routes from former patch.rs
    EndpointInfo { path: "/patch", method: "PATCH", description: "Echoes request details for PATCH, expects JSON body." },
    // Routes from former delete.rs
    EndpointInfo { path: "/delete", method: "DELETE", description: "Echoes request details for DELETE." },
    // Routes from former options.rs
    EndpointInfo { path: "/options", method: "OPTIONS", description: "Responds with allowed HTTP methods." },
    // Routes from former status.rs
    EndpointInfo { path: "/status/:code", method: "ANY", description: "Returns the specified HTTP status code." },
    // Routes from former anything.rs
    EndpointInfo { path: "/anything", method: "ANY", description: "Echoes request details for any HTTP method." },
    EndpointInfo { path: "/anything/*path", method: "ANY", description: "Echoes request details for any HTTP method under a specific path." },

    // Add the new entry for /endpoints itself
    EndpointInfo { path: "/endpoints", method: "GET", description: "Lists all available API endpoints." } 
];

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
        // Route for /endpoints
        .route("/endpoints", get(endpoints_handler))
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

// Handler for /endpoints
async fn endpoints_handler(
    Query(pretty_query): Query<PrettyQuery>,
) -> Response {
    let pretty = pretty_query.pretty.unwrap_or(false);
    
    match serde_json::to_value(API_ENDPOINTS) {
        Ok(json_value) => format_json_response(json_value, pretty),
        Err(_) => {
            format_error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to serialize endpoint data.")
        }
    }
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
