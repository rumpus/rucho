//! Metrics collection and storage for request statistics.
//!
//! This module provides thread-safe metrics tracking for:
//! - Total request counts (all time)
//! - Per-endpoint hit counts
//! - Success (2xx) vs failure (4xx/5xx) counts
//! - Rolling 1-hour window for all above metrics

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Number of buckets for the rolling window (one per minute for 60 minutes).
const ROLLING_WINDOW_BUCKETS: usize = 60;

/// Duration of each bucket in the rolling window.
const BUCKET_DURATION: Duration = Duration::from_secs(60);

/// A single time bucket for rolling window metrics.
#[derive(Debug, Default)]
struct TimeBucket {
    /// Timestamp when this bucket started (None if never used).
    start_time: Option<Instant>,
    /// Total requests in this bucket.
    requests: u64,
    /// Success responses (2xx) in this bucket.
    successes: u64,
    /// Failure responses (4xx/5xx) in this bucket.
    failures: u64,
    /// Per-endpoint counts in this bucket.
    endpoint_hits: HashMap<String, u64>,
}

impl TimeBucket {
    fn new() -> Self {
        Self::default()
    }

    fn reset(&mut self, start_time: Instant) {
        self.start_time = Some(start_time);
        self.requests = 0;
        self.successes = 0;
        self.failures = 0;
        self.endpoint_hits.clear();
    }

    fn is_expired(&self, now: Instant) -> bool {
        match self.start_time {
            Some(start) => now.duration_since(start) >= BUCKET_DURATION,
            None => true,
        }
    }

    fn is_within_window(&self, now: Instant, window: Duration) -> bool {
        match self.start_time {
            Some(start) => now.duration_since(start) < window,
            None => false,
        }
    }
}

/// Thread-safe metrics storage for request statistics.
///
/// Provides both all-time counters and rolling 1-hour window statistics.
pub struct Metrics {
    /// Total requests since server start.
    total_requests: AtomicU64,
    /// Total success responses (2xx) since server start.
    total_successes: AtomicU64,
    /// Total failure responses (4xx/5xx) since server start.
    total_failures: AtomicU64,
    /// Per-endpoint hit counts (all time).
    endpoint_hits: RwLock<HashMap<String, u64>>,
    /// Rolling window buckets for time-based statistics.
    rolling_buckets: RwLock<Vec<TimeBucket>>,
    /// Index of the current bucket being written to.
    current_bucket_idx: RwLock<usize>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// Creates a new Metrics instance with all counters initialized to zero.
    pub fn new() -> Self {
        let buckets: Vec<TimeBucket> = (0..ROLLING_WINDOW_BUCKETS)
            .map(|_| TimeBucket::new())
            .collect();
        Self {
            total_requests: AtomicU64::new(0),
            total_successes: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            endpoint_hits: RwLock::new(HashMap::new()),
            rolling_buckets: RwLock::new(buckets),
            current_bucket_idx: RwLock::new(0),
        }
    }

    /// Records a request to the metrics store.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint path that was requested (e.g., "/get", "/post")
    /// * `status_code` - The HTTP status code returned
    pub fn record_request(&self, endpoint: &str, status_code: u16) {
        let now = Instant::now();
        let is_success = (200..300).contains(&status_code);

        // Update all-time counters
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if is_success {
            self.total_successes.fetch_add(1, Ordering::Relaxed);
        } else if status_code >= 400 {
            self.total_failures.fetch_add(1, Ordering::Relaxed);
        }

        // Update all-time endpoint hits
        {
            let mut hits = self.endpoint_hits.write().unwrap();
            *hits.entry(endpoint.to_string()).or_insert(0) += 1;
        }

        // Update rolling window
        self.update_rolling_window(now, endpoint, is_success, status_code >= 400);
    }

    fn update_rolling_window(
        &self,
        now: Instant,
        endpoint: &str,
        is_success: bool,
        is_failure: bool,
    ) {
        let mut buckets = self.rolling_buckets.write().unwrap();
        let mut idx = self.current_bucket_idx.write().unwrap();

        // Check if current bucket is expired and we need to move to the next
        if buckets[*idx].is_expired(now) {
            *idx = (*idx + 1) % ROLLING_WINDOW_BUCKETS;
            buckets[*idx].reset(now);
        }

        // Record in current bucket
        let bucket = &mut buckets[*idx];
        bucket.requests += 1;
        if is_success {
            bucket.successes += 1;
        }
        if is_failure {
            bucket.failures += 1;
        }
        *bucket
            .endpoint_hits
            .entry(endpoint.to_string())
            .or_insert(0) += 1;
    }

    /// Returns all-time total request count.
    pub fn get_total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// Returns all-time success count.
    pub fn get_total_successes(&self) -> u64 {
        self.total_successes.load(Ordering::Relaxed)
    }

    /// Returns all-time failure count.
    pub fn get_total_failures(&self) -> u64 {
        self.total_failures.load(Ordering::Relaxed)
    }

    /// Returns all-time per-endpoint hit counts.
    pub fn get_endpoint_hits(&self) -> HashMap<String, u64> {
        self.endpoint_hits.read().unwrap().clone()
    }

    /// Returns request count for the last hour.
    pub fn get_last_hour_requests(&self) -> u64 {
        self.sum_rolling_window(|b| b.requests)
    }

    /// Returns success count for the last hour.
    pub fn get_last_hour_successes(&self) -> u64 {
        self.sum_rolling_window(|b| b.successes)
    }

    /// Returns failure count for the last hour.
    pub fn get_last_hour_failures(&self) -> u64 {
        self.sum_rolling_window(|b| b.failures)
    }

    /// Returns per-endpoint hit counts for the last hour.
    pub fn get_last_hour_endpoint_hits(&self) -> HashMap<String, u64> {
        let now = Instant::now();
        let window = Duration::from_secs(3600);
        let buckets = self.rolling_buckets.read().unwrap();

        let mut result: HashMap<String, u64> = HashMap::new();
        for bucket in buckets.iter() {
            if bucket.is_within_window(now, window) {
                for (endpoint, count) in &bucket.endpoint_hits {
                    *result.entry(endpoint.clone()).or_insert(0) += count;
                }
            }
        }
        result
    }

    fn sum_rolling_window<F>(&self, extractor: F) -> u64
    where
        F: Fn(&TimeBucket) -> u64,
    {
        let now = Instant::now();
        let window = Duration::from_secs(3600);
        let buckets = self.rolling_buckets.read().unwrap();

        buckets
            .iter()
            .filter(|b| b.is_within_window(now, window))
            .map(&extractor)
            .sum()
    }

    /// Returns a snapshot of all metrics as a serializable structure.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            all_time: AllTimeMetrics {
                total_requests: self.get_total_requests(),
                successes: self.get_total_successes(),
                failures: self.get_total_failures(),
                endpoint_hits: self.get_endpoint_hits(),
            },
            last_hour: LastHourMetrics {
                total_requests: self.get_last_hour_requests(),
                successes: self.get_last_hour_successes(),
                failures: self.get_last_hour_failures(),
                endpoint_hits: self.get_last_hour_endpoint_hits(),
            },
        }
    }
}

/// A serializable snapshot of all metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    /// All-time metrics since server start.
    pub all_time: AllTimeMetrics,
    /// Rolling metrics for the last hour.
    pub last_hour: LastHourMetrics,
}

/// All-time metrics since server start.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AllTimeMetrics {
    /// Total request count.
    pub total_requests: u64,
    /// Success response count (2xx).
    pub successes: u64,
    /// Failure response count (4xx/5xx).
    pub failures: u64,
    /// Per-endpoint hit counts.
    pub endpoint_hits: HashMap<String, u64>,
}

/// Rolling metrics for the last hour.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LastHourMetrics {
    /// Total request count in the last hour.
    pub total_requests: u64,
    /// Success response count (2xx) in the last hour.
    pub successes: u64,
    /// Failure response count (4xx/5xx) in the last hour.
    pub failures: u64,
    /// Per-endpoint hit counts in the last hour.
    pub endpoint_hits: HashMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_metrics_are_zero() {
        let metrics = Metrics::new();
        assert_eq!(metrics.get_total_requests(), 0);
        assert_eq!(metrics.get_total_successes(), 0);
        assert_eq!(metrics.get_total_failures(), 0);
        assert!(metrics.get_endpoint_hits().is_empty());
    }

    #[test]
    fn test_record_success_request() {
        let metrics = Metrics::new();
        metrics.record_request("/get", 200);

        assert_eq!(metrics.get_total_requests(), 1);
        assert_eq!(metrics.get_total_successes(), 1);
        assert_eq!(metrics.get_total_failures(), 0);
        assert_eq!(metrics.get_endpoint_hits().get("/get"), Some(&1));
    }

    #[test]
    fn test_record_failure_request() {
        let metrics = Metrics::new();
        metrics.record_request("/post", 500);

        assert_eq!(metrics.get_total_requests(), 1);
        assert_eq!(metrics.get_total_successes(), 0);
        assert_eq!(metrics.get_total_failures(), 1);
        assert_eq!(metrics.get_endpoint_hits().get("/post"), Some(&1));
    }

    #[test]
    fn test_record_client_error() {
        let metrics = Metrics::new();
        metrics.record_request("/invalid", 404);

        assert_eq!(metrics.get_total_requests(), 1);
        assert_eq!(metrics.get_total_successes(), 0);
        assert_eq!(metrics.get_total_failures(), 1);
    }

    #[test]
    fn test_multiple_endpoints() {
        let metrics = Metrics::new();
        metrics.record_request("/get", 200);
        metrics.record_request("/get", 200);
        metrics.record_request("/post", 201);
        metrics.record_request("/delete", 500);

        assert_eq!(metrics.get_total_requests(), 4);
        assert_eq!(metrics.get_total_successes(), 3);
        assert_eq!(metrics.get_total_failures(), 1);

        let hits = metrics.get_endpoint_hits();
        assert_eq!(hits.get("/get"), Some(&2));
        assert_eq!(hits.get("/post"), Some(&1));
        assert_eq!(hits.get("/delete"), Some(&1));
    }

    #[test]
    fn test_snapshot_structure() {
        let metrics = Metrics::new();
        metrics.record_request("/get", 200);
        metrics.record_request("/post", 500);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.all_time.total_requests, 2);
        assert_eq!(snapshot.all_time.successes, 1);
        assert_eq!(snapshot.all_time.failures, 1);
    }

    #[test]
    fn test_3xx_is_neither_success_nor_failure() {
        let metrics = Metrics::new();
        metrics.record_request("/redirect", 301);

        assert_eq!(metrics.get_total_requests(), 1);
        assert_eq!(metrics.get_total_successes(), 0);
        assert_eq!(metrics.get_total_failures(), 0);
    }
}
