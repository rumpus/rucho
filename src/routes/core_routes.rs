use axum::{
    routing::{get, post, put, patch, delete, options, head, any},
    Router,
    extract::{Json, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::utils::{
    json_response::format_json_response,
    error_response::format_error_response,
    request_models::PrettyQuery,
};
use utoipa::ToSchema;

/// Request payload wrapper for POST, PUT, PATCH, and DELETE handlers.
///
/// This newtype wraps a `serde_json::Value` to accept arbitrary JSON bodies
/// in requests that support request bodies.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Payload(serde_json::Value);

/// Serializes HTTP headers into a JSON object.
///
/// Converts an Axum `HeaderMap` into a `serde_json::Value` where each header
/// name becomes a key and its value becomes a string value. Invalid UTF-8
/// header values are replaced with `<invalid utf8>`.
///
/// # Arguments
///
/// * `headers` - Reference to the HeaderMap to serialize
///
/// # Returns
///
/// A `serde_json::Value` containing the headers as a JSON object
fn serialize_headers(headers: &HeaderMap) -> serde_json::Value {
    headers
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                serde_json::Value::String(v.to_str().unwrap_or("<invalid utf8>").to_string()),
            )
        })
        .collect::<serde_json::Map<_, _>>()
        .into()
}

/// Represents information about an API endpoint.
#[derive(Serialize, Debug, Clone, Copy, ToSchema)]
pub struct EndpointInfo {
    /// The path of the endpoint (e.g., "/get").
    #[schema(example = "/get")]
    path: &'static str,
    /// The HTTP method of the endpoint (e.g., "GET").
    #[schema(example = "GET")]
    method: &'static str,
    /// A brief description of the endpoint's purpose.
    #[schema(example = "Echoes request details for GET.")]
    description: &'static str,
}

/// A static array listing all core API endpoints provided by the server.
///
/// This array is used by the `/endpoints` handler to provide a discoverable list
/// of available API operations, including their paths, HTTP methods, and descriptions.
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

    // Health check endpoint
    EndpointInfo { path: "/healthz", method: "GET", description: "Performs a health check." },
    // Delay endpoint
    EndpointInfo {
        path: "/delay/:n",
        method: "ANY",
        description: "Delays the response by 'n' seconds. Replace :n with a number."
    },
    // Swagger UI endpoint
    EndpointInfo {
        path: "/swagger-ui",
        method: "GET",
        description: "Displays the OpenAPI/Swagger UI."
    },
    // Add the new entry for /endpoints itself
    EndpointInfo { path: "/endpoints", method: "GET", description: "Lists all available API endpoints." }
];

/// Creates and returns the Axum router for the core API endpoints.
///
/// This router includes routes for various HTTP methods (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD),
/// status code testing, an "anything" endpoint, and a listing of all available endpoints.
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

// Handler definitions moved before router()

// From status.rs
/// Responds with the HTTP status code specified in the path.
///
/// This handler allows testing of how a client handles different HTTP status codes.
/// It accepts any HTTP method.
///
/// # Path Parameters:
/// - `code`: The HTTP status code to return (e.g., 200, 404, 500).
///
/// # Responses:
/// - Returns the status code specified by the `code` path parameter.
/// - If an invalid `code` is provided (e.g., not a number or out of valid range),
///   it defaults to `400 Bad Request`.
#[utoipa::path(
    get, post, put, patch, delete, options, head, // Indicates this path works for all these methods
    path = "/status/{code}",
    params(
        ("code" = u16, Path, description = "HTTP status code to return")
    ),
    responses(
        (status = 200, description = "Returns the specified status code"),
        (status = 400, description = "Invalid status code provided")
        // Other status codes are returned directly as specified by `code`
    )
)]
pub async fn status_handler(axum::extract::Path(code): axum::extract::Path<u16>, _method: axum::http::Method) -> Response {
    StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST).into_response()
}

// From anything.rs
/// Echoes back details of the incoming request for any HTTP method.
///
/// This endpoint is useful for debugging and understanding how requests are processed.
/// It reflects the method, path, query parameters, headers, and body of the request.
///
/// # Query Parameters:
/// - `pretty` (optional, boolean): If `true`, the JSON response will be pretty-printed.
///
/// # Responses:
/// - `200 OK`: Successfully echoed the request details as a JSON object.
///
/// Note: While this handler is registered for `/anything` and `/anything/*path`,
/// the OpenAPI documentation for `/anything/*path` is handled by `anything_path_handler`
/// due to current limitations in `utoipa` with wildcard path parameters in a single handler.
#[utoipa::path(
    get, post, put, patch, delete, options, head, // Indicates this path works for all these methods
    path = "/anything",
    params(
        PrettyQuery
    ),
    responses(
        (status = 200, description = "Echoes request details", body = serde_json::Value)
    )
)]
pub async fn anything_handler(
    method: axum::http::Method,
    axum::extract::OriginalUri(uri): axum::extract::OriginalUri,
    headers: HeaderMap,
    Query(query): Query<PrettyQuery>,
    body: axum::body::Body
) -> impl IntoResponse {
    let pretty = query.pretty.unwrap_or(false);
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return format_json_response(json!({"error": "Failed to read body"}), pretty),
    };

    let resp = json!({
        "method": method.to_string(),
        "path": uri.path(),
        "query": uri.query().unwrap_or(""),
        "headers": serialize_headers(&headers),
        "body": String::from_utf8_lossy(&body_bytes),
    });

    format_json_response(resp, pretty)
}

#[utoipa::path(
    get, post, put, patch, delete, options, head,
    path = "/anything/{path:.*}",
    params(
        ("path" = String, Path, description = "Subpath for anything endpoint"),
        PrettyQuery
    ),
    responses(
        (status = 200, description = "Echoes request details for subpath", body = serde_json::Value)
    )
)]
#[allow(dead_code)] // To suppress warnings as it's not called directly by our code
/// **OpenAPI Documentation Handler for `/anything/*path`**.
///
/// This function exists *solely* to generate the correct OpenAPI documentation
/// for requests to `/anything/{path:.*}` (e.g., `/anything/foo/bar`).
/// The actual requests to these wildcard paths are handled by `anything_handler`.
///
/// This separation is necessary due to current limitations in `utoipa` regarding
/// the generation of OpenAPI specs for handlers that serve both a fixed path and a wildcard path.
///
/// # Path Parameters:
/// - `path`: The subpath captured by the wildcard (e.g., "foo/bar").
///
/// # Query Parameters:
/// - `pretty` (optional, boolean): If `true`, the JSON response will be pretty-printed.
///
/// # Responses:
/// - `200 OK`: Echoes request details for the subpath.
/// - **Note**: This handler, if ever called directly, returns `501 Not Implemented`.
pub async fn anything_path_handler(
    // Signature can mirror anything_handler but must include the Path extractor for "path"
    // utoipa needs to see axum::extract::Path here for the {path:.*} parameter.
    #[allow(unused_variables)] method: axum::http::Method,
    #[allow(unused_variables)] uri: axum::extract::OriginalUri,
    #[allow(unused_variables)] headers: axum::http::HeaderMap,
    #[allow(unused_variables)] query: axum::extract::Query<PrettyQuery>,
    #[allow(unused_variables)] path_param: axum::extract::Path<String>, // This is key for utoipa
    #[allow(unused_variables)] body: axum::body::Body
) -> Response { // Changed to concrete Response type
    // This function body is not intended to be executed.
    // The actual logic for "/anything/*" paths is in `anything_handler`.
    // This exists for OpenAPI generation purposes.
    (axum::http::StatusCode::NOT_IMPLEMENTED, "This handler is only for OpenAPI documentation of /anything/*path. The actual requests are handled by `anything_handler`.".to_string()).into_response()
}

// From get.rs
/// Serves a welcome message at the root path (`/`).
///
/// # HTTP Method:
/// - `GET`
///
/// # Responses:
/// - `200 OK`: Returns a plain text welcome message.
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Welcome message", body = String)
    )
)]
pub async fn root_handler() -> &'static str {
    "Welcome to Echo Server!
"
}

/// Handles GET requests to `/get`.
///
/// Echoes back the request's method and headers as a JSON object.
///
/// # HTTP Method:
/// - `GET`
///
/// # Query Parameters:
/// - `pretty` (optional, boolean): If `true`, the JSON response will be pretty-printed.
///
/// # Responses:
/// - `200 OK`: Returns a JSON object containing the method and headers.
#[utoipa::path(
    get,
    path = "/get",
    params(
        PrettyQuery
    ),
    responses(
        (status = 200, description = "Echoes request details", body = serde_json::Value)
    )
)]
pub async fn get_handler(
    headers: HeaderMap,
    Query(pretty_query): Query<PrettyQuery>,
) -> Response {
    let pretty = pretty_query.pretty.unwrap_or(false);
    let payload = json!({
        "method": "GET",
        "headers": serialize_headers(&headers),
    });
    format_json_response(payload, pretty)
}

/// Handles HEAD requests to `/get`.
///
/// Responds with the same headers as a GET request to `/get`, but with no body.
/// This is typically used to check if a resource exists or to get its metadata
/// without transferring the entire content.
///
/// # HTTP Method:
/// - `HEAD`
///
/// # Responses:
/// - `200 OK`: Returns an empty body with appropriate headers.
#[utoipa::path(
    head,
    path = "/get",
    responses(
        (status = 200, description = "Responds with headers for GET query")
    )
)]
pub async fn head_handler() -> impl IntoResponse {
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .body(axum::body::Body::empty())
        .unwrap()
}

// Handler for /endpoints
/// Lists all available API endpoints provided by this server.
///
/// Returns a JSON array of `EndpointInfo` objects, each describing an endpoint's
/// path, HTTP method, and a brief description.
///
/// # HTTP Method:
/// - `GET`
///
/// # Query Parameters:
/// - `pretty` (optional, boolean): If `true`, the JSON response will be pretty-printed.
///
/// # Responses:
/// - `200 OK`: Successfully returns the list of endpoints.
/// - `500 Internal Server Error`: If there's an issue serializing the endpoint data.
#[utoipa::path(
    get,
    path = "/endpoints",
    params(
        PrettyQuery
    ),
    responses(
        (status = 200, description = "Lists all available API endpoints", body = Vec<EndpointInfo>),
        (status = 500, description = "Failed to serialize endpoint data")
    )
)]
pub async fn endpoints_handler(
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

// From post.rs
/// Handles POST requests to `/post`.
///
/// Echoes back the request's method, headers, and the parsed JSON body.
/// Expects a JSON request body.
///
/// # HTTP Method:
/// - `POST`
///
/// # Query Parameters:
/// - `pretty` (optional, boolean): If `true`, the JSON response will be pretty-printed.
///
/// # Request Body:
/// - `Payload`: A generic JSON object.
///
/// # Responses:
/// - `200 OK`: Returns a JSON object containing method, headers, and parsed body.
/// - `400 Bad Request`: If the request body is not valid JSON.
#[utoipa::path(
    post,
    path = "/post",
    params(
        PrettyQuery
    ),
    request_body = Payload,
    responses(
        (status = 200, description = "Echoes request details", body = serde_json::Value),
        (status = 400, description = "Invalid JSON payload")
    )
)]
pub async fn post_handler(
    headers: HeaderMap,
    Query(pretty_query): Query<PrettyQuery>,
    body: Result<Json<serde_json::Value>, axum::extract::rejection::JsonRejection>
) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);
    match body {
        Ok(Json(payload_value)) => {
            let response_payload = json!({
                "method": "POST",
                "headers": serialize_headers(&headers),
                "body": payload_value,
            });
            format_json_response(response_payload, pretty)
        }
        Err(_) => format_error_response(StatusCode::BAD_REQUEST, "Invalid JSON payload")
    }
}

// From put.rs
/// Handles PUT requests to `/put`.
///
/// Echoes back the request's method, headers, and the parsed JSON body.
/// Expects a JSON request body.
///
/// # HTTP Method:
/// - `PUT`
///
/// # Query Parameters:
/// - `pretty` (optional, boolean): If `true`, the JSON response will be pretty-printed.
///
/// # Request Body:
/// - `Payload`: A generic JSON object.
///
/// # Responses:
/// - `200 OK`: Returns a JSON object containing method, headers, and parsed body.
/// - `400 Bad Request`: If the request body is not valid JSON.
#[utoipa::path(
    put,
    path = "/put",
    params(
        PrettyQuery
    ),
    request_body = Payload,
    responses(
        (status = 200, description = "Echoes request details", body = serde_json::Value),
        (status = 400, description = "Invalid JSON payload")
    )
)]
pub async fn put_handler(
    headers: HeaderMap,
    Query(pretty_query): Query<PrettyQuery>,
    body: Result<Json<Payload>, axum::extract::rejection::JsonRejection>
) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);
    match body {
        Ok(Json(Payload(body_json))) => {
            let payload = json!({
                "method": "PUT",
                "headers": serialize_headers(&headers),
                "body": body_json,
            });
            format_json_response(payload, pretty)
        }
        Err(_) => format_error_response(StatusCode::BAD_REQUEST, "Invalid JSON payload")
    }
}

// From patch.rs
/// Handles PATCH requests to `/patch`.
///
/// Echoes back the request's method, headers, and the parsed JSON body.
/// Expects a JSON request body.
///
/// # HTTP Method:
/// - `PATCH`
///
/// # Query Parameters:
/// - `pretty` (optional, boolean): If `true`, the JSON response will be pretty-printed.
///
/// # Request Body:
/// - `Payload`: A generic JSON object.
///
/// # Responses:
/// - `200 OK`: Returns a JSON object containing method, headers, and parsed body.
/// - `400 Bad Request`: If the request body is not valid JSON.
#[utoipa::path(
    patch,
    path = "/patch",
    params(
        PrettyQuery
    ),
    request_body = Payload,
    responses(
        (status = 200, description = "Echoes request details", body = serde_json::Value),
        (status = 400, description = "Invalid JSON payload")
    )
)]
pub async fn patch_handler(
    headers: HeaderMap,
    Query(pretty_query): Query<PrettyQuery>,
    body: Result<Json<Payload>, axum::extract::rejection::JsonRejection>
) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);
    match body {
        Ok(Json(Payload(body_json))) => {
            let payload = json!({
                "method": "PATCH",
                "headers": serialize_headers(&headers),
                "body": body_json,
            });
            format_json_response(payload, pretty)
        }
        Err(_) => format_error_response(StatusCode::BAD_REQUEST, "Invalid JSON payload")
    }
}

// From delete.rs
/// Handles DELETE requests to `/delete`.
///
/// Echoes back the request's method and headers. If a JSON body is provided,
/// it will also be echoed. Otherwise, the body in the response will be `null`.
///
/// # HTTP Method:
/// - `DELETE`
///
/// # Query Parameters:
/// - `pretty` (optional, boolean): If `true`, the JSON response will be pretty-printed.
///
/// # Request Body:
/// - `Payload` (optional): A generic JSON object.
///
/// # Responses:
/// - `200 OK`: Returns a JSON object containing method, headers, and body (if provided).
#[utoipa::path(
    delete,
    path = "/delete",
    params(
        PrettyQuery
    ),
    request_body = Option<Payload>, // Indicates optional body
    responses(
        (status = 200, description = "Echoes request details, body is null if not provided", body = serde_json::Value)
    )
)]
pub async fn delete_handler(
    headers: HeaderMap,
    Query(pretty_query): Query<PrettyQuery>,
    // Axum's Json extractor requires the body to be valid JSON if Content-Type: application/json is sent.
    // To make the body truly optional even with Content-Type, we'd need a custom extractor or to read the body manually.
    // For now, if Content-Type: application/json is sent, a valid JSON body (e.g. "{}") is expected or it's a rejection.
    // If no Content-Type or a different one is sent, `body` will likely be an Err.
    body: Result<Json<Payload>, axum::extract::rejection::JsonRejection>
) -> impl IntoResponse {
    let pretty = pretty_query.pretty.unwrap_or(false);
    match body {
        Ok(Json(Payload(body_json))) => {
            let payload = json!({
                "method": "DELETE",
                "headers": serialize_headers(&headers),
                "body": body_json,
            });
            format_json_response(payload, pretty)
        }
        Err(_) => {
            let payload = json!({
                "method": "DELETE",
                "headers": serialize_headers(&headers),
                "body": serde_json::Value::Null,
            });
            format_json_response(payload, pretty)
        }
    }
}

// From options.rs
/// Handles OPTIONS requests to `/options`.
///
/// Responds with the allowed HTTP methods for this server in the `Allow` header.
/// The body of the response is empty.
///
/// # HTTP Method:
/// - `OPTIONS`
///
/// # Responses:
/// - `204 No Content`: Returns an empty body with the `Allow` header set to
///   "GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD".
#[utoipa::path(
    options,
    path = "/options",
    responses(
        (status = 204, description = "No content, Allow header lists allowed methods")
    )
)]
pub async fn options_handler() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header(axum::http::header::ALLOW, "GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD")
        .body(axum::body::Body::empty())
        .unwrap()
}
