//! Request timing utilities.
//!
//! This module provides types for tracking request timing information
//! that can be injected into responses.

use std::time::Instant;

/// Stores the start time of a request for timing calculations.
///
/// This struct is inserted as a request extension by the timing middleware
/// and can be extracted by handlers to calculate request duration.
#[derive(Clone, Copy)]
pub struct RequestTiming {
    /// The instant when the request started processing.
    pub start: Instant,
}

impl RequestTiming {
    /// Creates a new RequestTiming with the current instant.
    pub fn now() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Returns the elapsed time since the request started in milliseconds.
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}
