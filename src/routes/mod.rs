//! HTTP route handlers for the Rucho web server.
//!
//! This module contains all the HTTP route handlers organized into submodules:
//!
//! - [`base64`] - Base64 decoding endpoint
//! - [`bytes`] - Random bytes endpoint
//! - [`cookies`] - Cookie inspection and manipulation endpoints
//! - [`core_routes`] - Main API endpoints (GET, POST, PUT, PATCH, DELETE, etc.)
//! - [`delay`] - Delay endpoint for testing timeouts
//! - [`drip`] - Slow-streaming bytes endpoint for testing inter-byte timeouts
//! - [`healthz`] - Health check endpoint
//! - [`metrics`] - Metrics endpoint (JSON)
//! - [`redirect`] - Chained redirect endpoint
//! - [`response_headers`] - Echo query params as response headers

/// Module for the base64 decoding endpoint (`/base64/:encoded`).
pub mod base64;
/// Module for the random-bytes endpoint (`/bytes/:n`).
pub mod bytes;
/// Module for the cookie inspection and manipulation endpoints (`/cookies`).
pub mod cookies;
/// Module for core API routes, including various HTTP method handlers and utility endpoints.
pub mod core_routes;
/// Module for the delay endpoint (`/delay/:n`).
pub mod delay;
/// Module for the slow-streaming drip endpoint (`/drip`).
pub mod drip;
/// Module for the health check endpoint (`/healthz`).
pub mod healthz;
/// Module for the metrics endpoint (`/metrics`).
pub mod metrics;
/// Module for the redirect endpoint (`/redirect/:n`).
pub mod redirect;
/// Module for the response-headers endpoint (`/response-headers`).
pub mod response_headers;
