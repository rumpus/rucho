// Make each route module public so they can be used elsewhere in the project

/// Module for core API routes, including various HTTP method handlers and utility endpoints.
pub mod core_routes; // Consolidated routes
/// Module for the health check endpoint (`/healthz`).
pub mod healthz;
/// Module for the delay endpoint (`/delay/:n`).
pub mod delay;
