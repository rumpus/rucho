//! HTTP route handlers for the Rucho web server.
//!
//! This module contains all the HTTP route handlers organized into submodules:
//!
//! - [`core_routes`] - Main API endpoints (GET, POST, PUT, PATCH, DELETE, etc.)
//! - [`delay`] - Delay endpoint for testing timeouts
//! - [`healthz`] - Health check endpoint

/// Module for core API routes, including various HTTP method handlers and utility endpoints.
pub mod core_routes;
/// Module for the delay endpoint (`/delay/:n`).
pub mod delay;
/// Module for the health check endpoint (`/healthz`).
pub mod healthz;
/// Module for the metrics endpoint (`/metrics`).
pub mod metrics;
