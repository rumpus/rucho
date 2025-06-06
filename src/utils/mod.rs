/// # Utilities Module
///
/// This module aggregates and declares various utility functions and structures
/// used throughout the application. It re-exports sub-modules containing
/// specific utilities like configuration management, response formatting, etc.
// Make the `json_response` module public
// This allows other parts of the project (like main.rs and route handlers) to use `utils::json_response::format_json_response`

/// Module for application configuration loading and management.
pub mod config; // Added
/// Module for creating standardized JSON error responses.
pub mod error_response;
/// Module for creating standardized JSON responses.
pub mod json_response;
/// Module defining common request model structures, like query parameters.
pub mod request_models;
/// Module for server-specific configurations, including listener parsing and SSL setup.
pub mod server_config;
