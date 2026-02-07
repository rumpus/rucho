//! Chaos engineering middleware layer.
//!
//! This module provides middleware that randomly injects failures, delays, and
//! response corruption to help test application resilience. Each chaos type
//! rolls independently against its configured probability rate per request.

use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use http::StatusCode;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;

use crate::utils::config::ChaosConfig;

/// Middleware that injects chaos behaviors based on configuration.
///
/// Evaluation order: failure → delay → corruption.
/// Failure short-circuits (skips handler). Delay and corruption can stack.
/// When `inform_header` is true, affected responses include an `X-Chaos` header
/// listing which chaos types were applied.
pub async fn chaos_middleware(
    request: Request,
    next: Next,
    chaos: Arc<ChaosConfig>,
) -> Response<Body> {
    let mut rng = StdRng::from_entropy();
    let mut applied: Vec<&str> = Vec::new();

    // 1. Roll for failure — short-circuit with error response
    if chaos.has_failure() && rng.gen::<f64>() < chaos.failure_rate {
        let code_idx = rng.gen_range(0..chaos.failure_codes.len());
        let status_code = chaos.failure_codes[code_idx];
        applied.push("failure");

        let body = serde_json::json!({
            "error": "Chaos failure injected",
            "chaos": {
                "type": "failure",
                "status_code": status_code
            }
        });

        let mut response = Response::builder()
            .status(StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string_pretty(&body).unwrap()))
            .unwrap();

        if chaos.inform_header {
            response
                .headers_mut()
                .insert("x-chaos", applied.join(",").parse().unwrap());
        }

        return response;
    }

    // 2. Roll for delay — sleep before passing to handler
    if chaos.has_delay() && rng.gen::<f64>() < chaos.delay_rate {
        let delay_ms = if chaos.delay_ms == "random" {
            rng.gen_range(0..chaos.delay_max_ms)
        } else {
            chaos.delay_ms.parse::<u64>().unwrap_or(0)
        };

        if delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }
        applied.push("delay");
    }

    // 3. Call the inner handler
    let response = next.run(request).await;

    // 4. Roll for corruption — modify response body
    if chaos.has_corruption() && rng.gen::<f64>() < chaos.corruption_rate {
        applied.push("corruption");
        let (mut parts, body) = response.into_parts();

        let corrupted_body = match chaos.corruption_type.as_str() {
            "empty" => Body::empty(),
            "truncate" => {
                let bytes = axum::body::to_bytes(body, usize::MAX)
                    .await
                    .unwrap_or_default();
                let half = bytes.len() / 2;
                Body::from(bytes.slice(0..half))
            }
            "garbage" => {
                let bytes = axum::body::to_bytes(body, usize::MAX)
                    .await
                    .unwrap_or_default();
                let len = bytes.len();
                let garbage: Vec<u8> = (0..len).map(|_| rng.gen_range(0x21..0x7F)).collect();
                Body::from(garbage)
            }
            _ => body, // Shouldn't happen after validation
        };

        // 5. Add X-Chaos header if inform_header enabled and any effect applied
        if chaos.inform_header && !applied.is_empty() {
            parts
                .headers
                .insert("x-chaos", applied.join(",").parse().unwrap());
        }

        return Response::from_parts(parts, corrupted_body);
    }

    // 5. Add X-Chaos header if inform_header enabled and any effect applied (no corruption path)
    if chaos.inform_header && !applied.is_empty() {
        let (mut parts, body) = response.into_parts();
        parts
            .headers
            .insert("x-chaos", applied.join(",").parse().unwrap());
        return Response::from_parts(parts, body);
    }

    response
}
