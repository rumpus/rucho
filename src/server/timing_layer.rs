//! Request timing middleware layer.
//!
//! This module provides middleware that captures the start time of each request
//! and makes it available to handlers via request extensions.

use axum::{body::Body, extract::Request, middleware::Next, response::Response};

use crate::utils::timing::RequestTiming;

/// Middleware function that captures request start time.
///
/// This middleware inserts a `RequestTiming` struct into the request extensions
/// at the very beginning of request processing. Handlers can extract this
/// to calculate how long the request took to process.
pub async fn timing_middleware(mut request: Request, next: Next) -> Response<Body> {
    // Capture start time and insert into request extensions
    request.extensions_mut().insert(RequestTiming::now());

    // Call the inner handler
    next.run(request).await
}
