//! Metrics endpoint for request statistics.
//!
//! This module provides the `/metrics` endpoint that returns JSON statistics
//! about server request activity.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::utils::metrics::Metrics;

/// Handler for the `/metrics` endpoint.
///
/// Returns a JSON object containing:
/// - `all_time`: Total requests, successes, failures, and per-endpoint hits since server start
/// - `last_hour`: Same metrics but only for the last 60 minutes (rolling window)
///
/// # Example Response
///
/// ```json
/// {
///   "all_time": {
///     "total_requests": 1000,
///     "successes": 950,
///     "failures": 50,
///     "endpoint_hits": {
///       "/get": 500,
///       "/post": 300,
///       "/status/:code": 200
///     }
///   },
///   "last_hour": {
///     "total_requests": 100,
///     "successes": 95,
///     "failures": 5,
///     "endpoint_hits": {
///       "/get": 50,
///       "/post": 30,
///       "/status/:code": 20
///     }
///   }
/// }
/// ```
pub async fn get_metrics(State(metrics): State<Arc<Metrics>>) -> impl IntoResponse {
    let snapshot = metrics.snapshot();
    (StatusCode::OK, Json(snapshot))
}
