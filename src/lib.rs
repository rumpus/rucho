//! Rucho - A lightweight HTTP/TCP/UDP echo server for testing and debugging.
//!
//! Rucho provides a simple server that echoes back request details, making it useful
//! for testing API clients, debugging network issues, and learning about HTTP protocols.
//!
//! # Features
//!
//! - HTTP/HTTPS endpoints that echo request details (headers, body, method, etc.)
//! - TCP and UDP echo servers for raw socket testing
//! - Configurable via files or environment variables
//! - OpenAPI/Swagger documentation
//!
//! # Architecture
//!
//! The crate is organized into the following modules:
//!
//! - [`routes`] - HTTP route handlers for various endpoints
//! - [`utils`] - Utility functions for configuration, response formatting, etc.
//! - [`tcp_udp_handlers`] - Raw TCP and UDP connection handlers
//! - [`cli`] - Command-line interface argument parsing and command handlers
//! - [`server`] - Server setup and orchestration

/// Command-line interface module for argument parsing and command handling.
pub mod cli;

/// The `routes` module contains all the route handlers for the Rucho web server.
///
/// This module defines the API endpoints and their corresponding logic.
pub mod routes;

/// Server setup and orchestration module.
///
/// Provides functionality for setting up HTTP/HTTPS, TCP, and UDP listeners.
pub mod server;

/// The `tcp_udp_handlers` module provides handlers for raw TCP and UDP connections.
///
/// These handlers implement simple echo functionality for testing network connectivity.
pub mod tcp_udp_handlers;

/// The `utils` module provides utility functions and structures used throughout the Rucho application.
///
/// This includes configuration management, server setup, and other helper functionalities.
pub mod utils;
