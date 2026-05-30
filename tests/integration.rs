//! Integration tests for Rucho HTTP endpoints.
//!
//! Each test spins up a real HTTP server on a random port and hits it with
//! `reqwest`, validating status codes, headers, and response bodies over
//! actual TCP connections.

use axum::{extract::DefaultBodyLimit, middleware, Router};
use rucho::routes::{
    base64, bytes, cache, content_types, cookies, core_routes, delay, drip, encoding, healthz,
    image, range, redirect, response_headers,
};
use rucho::server::timing_layer::timing_middleware;
use rucho::utils::constants::DEFAULT_MAX_BODY_SIZE_BYTES;

/// Spawns a test server on a random port and returns its base URL.
///
/// Builds a minimal Router (no chaos, metrics, tracing, or swagger) and
/// serves it on `127.0.0.1:0` so each test gets its own isolated server.
async fn spawn_app() -> String {
    spawn_app_with_body_limit(DEFAULT_MAX_BODY_SIZE_BYTES).await
}

/// Variant of `spawn_app` that caps request bodies at `max_body_size` bytes.
async fn spawn_app_with_body_limit(max_body_size: usize) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let app = Router::new()
        .merge(core_routes::router())
        .merge(healthz::router())
        .merge(delay::router())
        .merge(redirect::router())
        .merge(cookies::router())
        .merge(base64::router())
        .merge(bytes::router())
        .merge(cache::router())
        .merge(drip::router())
        .merge(encoding::router())
        .merge(response_headers::router())
        .merge(content_types::router())
        .merge(image::router())
        .merge(range::router())
        .layer(DefaultBodyLimit::max(max_body_size))
        .layer(middleware::from_fn(timing_middleware));

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap()
    });

    format!("http://{addr}")
}

/// Spawns a test server using the REAL `build_app()` — the full middleware stack
/// (metrics, chaos-gate, timing, trace, compression, CORS, normalize-path) wired
/// exactly as the binary wires it. Unlike `spawn_app()` (a minimal router), this
/// catches middleware-interaction regressions. Metrics are force-enabled so the
/// `/metrics` endpoint and its collection middleware are exercised.
async fn spawn_full_app() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let config = rucho::utils::config::Config::default();
    let metrics = Some(std::sync::Arc::new(rucho::utils::metrics::Metrics::new()));
    let chaos = std::sync::Arc::new(config.chaos.clone());
    let app = rucho::app::build_app(
        metrics,
        config.compression_enabled,
        chaos,
        config.max_body_size_bytes,
        config.request_id_enabled,
    );

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap()
    });

    format!("http://{addr}")
}

#[tokio::test]
async fn test_healthz() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/healthz")).await.unwrap();

    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert_eq!(body, "OK");
}

#[tokio::test]
async fn test_get_echo() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/get")).await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["method"], "GET");
    assert!(body["headers"].is_object());
}

#[tokio::test]
async fn test_x_response_time_header() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/get")).await.unwrap();

    assert_eq!(resp.status(), 200);
    let rt = resp
        .headers()
        .get("x-response-time")
        .expect("every response must carry x-response-time")
        .to_str()
        .unwrap();
    assert!(rt.ends_with("ms"), "expected a <n>ms value, got: {rt}");
    let ms: f64 = rt.trim_end_matches("ms").parse().expect("numeric ms");
    assert!(ms >= 0.0);
}

#[tokio::test]
async fn test_get_echoes_http_version() {
    // reqwest speaks HTTP/1.1 over plaintext, so the echo must reflect that.
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/get")).await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["http_version"], "HTTP/1.1");
}

#[tokio::test]
async fn test_anything_echoes_http_version() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/anything")).await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["method"], "GET");
    assert_eq!(body["http_version"], "HTTP/1.1");
}

#[tokio::test]
async fn test_post_echoes_http_version() {
    let base = spawn_app().await;
    let resp = reqwest::Client::new()
        .post(format!("{base}/post"))
        .json(&serde_json::json!({"k": "v"}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["http_version"], "HTTP/1.1");
}

#[tokio::test]
async fn test_post_echo() {
    let base = spawn_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base}/post"))
        .json(&serde_json::json!({"key": "value"}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["method"], "POST");
    assert_eq!(body["body"]["key"], "value");
}

#[tokio::test]
async fn test_put_echo() {
    let base = spawn_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .put(format!("{base}/put"))
        .json(&serde_json::json!({"update": true}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["method"], "PUT");
    assert_eq!(body["body"]["update"], true);
}

#[tokio::test]
async fn test_uuid() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/uuid")).await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let uuid_str = body["uuid"].as_str().expect("uuid should be a string");
    // Validate UUID v4 format (8-4-4-4-12 hex digits)
    assert!(
        uuid::Uuid::parse_str(uuid_str).is_ok(),
        "should be a valid UUID: {uuid_str}"
    );
}

#[tokio::test]
async fn test_ip() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/ip")).await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let origin = body["origin"].as_str().expect("origin should be a string");
    // With ConnectInfo fallback, the peer address (127.0.0.1 or ::1) should
    // appear even when no X-Forwarded-For / X-Real-IP header is set.
    assert_ne!(origin, "unknown", "origin should fall back to peer address");
    assert!(
        origin.parse::<std::net::IpAddr>().is_ok(),
        "origin should be a valid IP: {origin}"
    );
}

#[tokio::test]
async fn test_ip_respects_forwarded_header() {
    let base = spawn_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/ip"))
        .header("x-forwarded-for", "203.0.113.42, 10.0.0.1")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["origin"], "203.0.113.42");
}

#[tokio::test]
async fn test_headers() {
    let base = spawn_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/headers"))
        .header("x-custom-test", "hello-rucho")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["headers"]["x-custom-test"], "hello-rucho");
}

#[tokio::test]
async fn test_user_agent() {
    let base = spawn_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/user-agent"))
        .header("user-agent", "rucho-integration-test/1.0")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["user-agent"], "rucho-integration-test/1.0");
}

#[tokio::test]
async fn test_status_codes() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/status/418")).await.unwrap();
    assert_eq!(resp.status(), 418);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], 418);
    assert_eq!(body["reason"], "I'm a teapot");
}

#[tokio::test]
async fn test_redirect_chain() {
    let base = spawn_app().await;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let resp = client
        .get(format!("{base}/redirect/3"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 302);
    assert_eq!(
        resp.headers().get("location").unwrap().to_str().unwrap(),
        "/redirect/2"
    );
    assert_eq!(
        resp.headers()
            .get("x-redirect-count")
            .unwrap()
            .to_str()
            .unwrap(),
        "3"
    );
}

#[tokio::test]
async fn test_cookies_roundtrip() {
    let base = spawn_app().await;
    let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let client = reqwest::Client::builder()
        .cookie_provider(jar)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    // Set a cookie via /cookies/set
    let resp = client
        .get(format!("{base}/cookies/set?foo=bar"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 302);

    // The Set-Cookie header should have been captured by the jar.
    // Now hit /cookies — reqwest will send the cookie back automatically.
    let resp = client.get(format!("{base}/cookies")).send().await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["cookies"]["foo"], "bar");
}

#[tokio::test]
async fn test_base64_decode() {
    let base = spawn_app().await;
    // "Hello, Rucho!" -> SGVsbG8sIFJ1Y2hvIQ==
    let resp = reqwest::get(format!("{base}/base64/SGVsbG8sIFJ1Y2hvIQ=="))
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["decoded"], "Hello, Rucho!");
    assert_eq!(body["is_utf8"], true);
    assert_eq!(body["byte_length"], 13);
}

#[tokio::test]
async fn test_base64_invalid_returns_400() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/base64/a")).await.unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_anything_wildcard() {
    let base = spawn_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base}/anything/foo/bar"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["method"], "POST");
    assert_eq!(body["path"], "/anything/foo/bar");
}

#[tokio::test]
async fn test_bytes_returns_correct_size_and_type() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/bytes/512")).await.unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "application/octet-stream"
    );
    let body = resp.bytes().await.unwrap();
    assert_eq!(body.len(), 512);
}

#[tokio::test]
async fn test_bytes_exceeds_max_returns_400() {
    let base = spawn_app().await;
    // 10 MiB + 1 is one over the cap.
    let resp = reqwest::get(format!("{base}/bytes/{}", 10 * 1024 * 1024 + 1))
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_response_headers_mirrors_query_params() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!(
        "{base}/response-headers?x-rate-limit=100&cache-control=no-store"
    ))
    .await
    .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers().get("x-rate-limit").unwrap(), "100");
    assert_eq!(resp.headers().get("cache-control").unwrap(), "no-store");

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["x-rate-limit"], "100");
    assert_eq!(body["cache-control"], "no-store");
}

#[tokio::test]
async fn test_response_headers_invalid_returns_400() {
    let base = spawn_app().await;
    // %0A is a newline — invalid inside a header name.
    let resp = reqwest::get(format!("{base}/response-headers?bad%0Aname=value"))
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_drip_streams_correct_byte_count() {
    let base = spawn_app().await;
    // Tight duration — keeps the test fast while still going through the
    // streaming path.
    let resp = reqwest::get(format!("{base}/drip?numbytes=20&duration=0"))
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "application/octet-stream"
    );
    let body = resp.bytes().await.unwrap();
    assert_eq!(body.len(), 20);
    assert!(body.iter().all(|&b| b == b'*'));
}

#[tokio::test]
async fn test_drip_takes_at_least_requested_duration() {
    let base = spawn_app().await;
    let start = std::time::Instant::now();
    let resp = reqwest::get(format!("{base}/drip?numbytes=4&duration=1"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.bytes().await.unwrap();
    let elapsed = start.elapsed();
    assert_eq!(body.len(), 4);
    assert!(
        elapsed >= std::time::Duration::from_millis(950),
        "expected >= 950ms, got {elapsed:?}"
    );
}

#[tokio::test]
async fn test_drip_numbytes_cap_returns_400() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/drip?numbytes=10001"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_drip_custom_status_code() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/drip?numbytes=3&duration=0&code=504"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 504);
    let body = resp.bytes().await.unwrap();
    assert_eq!(body.len(), 3);
}

#[tokio::test]
async fn test_anything_body_limit_returns_413() {
    let base = spawn_app_with_body_limit(1024).await;
    let client = reqwest::Client::new();
    let big_body = vec![b'x'; 2048];
    let resp = client
        .post(format!("{base}/anything"))
        .body(big_body)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 413);
}

#[tokio::test]
async fn test_xml_returns_application_xml() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/xml")).await.unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "application/xml"
    );
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("<?xml"));
    assert!(body.contains("<rucho>"));
}

#[tokio::test]
async fn test_html_returns_text_html() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/html")).await.unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "text/html; charset=utf-8"
    );
    let body = resp.text().await.unwrap();
    assert!(body.contains("<!DOCTYPE html>"));
    assert!(body.contains("<h1>Rucho</h1>"));
}

#[tokio::test]
async fn test_image_png_returns_valid_png() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/image/png")).await.unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "image/png"
    );
    let body = resp.bytes().await.unwrap();
    // PNG magic number survives the round trip.
    assert_eq!(&body[..8], b"\x89PNG\r\n\x1a\n");
}

#[tokio::test]
async fn test_image_svg_returns_svg_xml() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/image/svg")).await.unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "image/svg+xml"
    );
    assert!(resp.text().await.unwrap().contains("<svg"));
}

#[tokio::test]
async fn test_image_unsupported_format_returns_400() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/image/gif")).await.unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_range_full_body_when_no_range_header() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/range/26")).await.unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers().get(reqwest::header::ACCEPT_RANGES).unwrap(),
        "bytes"
    );
    assert_eq!(resp.text().await.unwrap(), "abcdefghijklmnopqrstuvwxyz");
}

#[tokio::test]
async fn test_range_partial_content() {
    let base = spawn_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/range/26"))
        .header(reqwest::header::RANGE, "bytes=0-4")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 206);
    assert_eq!(
        resp.headers().get(reqwest::header::CONTENT_RANGE).unwrap(),
        "bytes 0-4/26"
    );
    assert_eq!(resp.text().await.unwrap(), "abcde");
}

#[tokio::test]
async fn test_range_unsatisfiable_returns_416() {
    let base = spawn_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/range/26"))
        .header(reqwest::header::RANGE, "bytes=100-200")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 416);
}

// --- Full-app (real build_app) regression tests ---

#[tokio::test]
async fn test_full_app_serves_metrics() {
    let base = spawn_full_app().await;
    let resp = reqwest::get(format!("{base}/metrics")).await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["all_time"]["total_requests"].is_number());
    assert!(body["last_hour"].is_object());
}

#[tokio::test]
async fn test_full_app_metrics_middleware_records_requests() {
    // The minimal spawn_app() omits the metrics middleware entirely, so this
    // regression is only observable through the real build_app() stack.
    let base = spawn_full_app().await;

    // Make a recorded request, then read the metrics.
    let _ = reqwest::get(format!("{base}/get")).await.unwrap();
    let body: serde_json::Value = reqwest::get(format!("{base}/metrics"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(
        body["all_time"]["total_requests"].as_u64().unwrap_or(0) >= 1,
        "metrics middleware did not record any request: {body}"
    );
    assert!(
        body["all_time"]["endpoint_hits"]["/get"]
            .as_u64()
            .unwrap_or(0)
            >= 1,
        "metrics middleware did not record the /get hit: {body}"
    );
}

#[tokio::test]
async fn test_full_app_echo_works_through_full_stack() {
    // Basic echo must still work with the whole middleware stack in place.
    let base = spawn_full_app().await;
    let resp = reqwest::get(format!("{base}/get")).await.unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["method"], "GET");
}

// --- Request-ID middleware (X-Request-Id) ---

/// Returns true if `s` has the canonical 8-4-4-4-12 hex UUID shape.
///
/// The integration crate can't depend on the `uuid` crate (it's a normal, not
/// dev, dependency), so we validate the shape structurally instead of parsing.
fn looks_like_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 5
        && [8usize, 4, 4, 4, 12]
            .iter()
            .zip(&parts)
            .all(|(&len, part)| part.len() == len && part.bytes().all(|b| b.is_ascii_hexdigit()))
}

#[tokio::test]
async fn test_request_id_present_and_uuid_when_absent() {
    let base = spawn_full_app().await;
    let resp = reqwest::get(format!("{base}/get")).await.unwrap();

    assert_eq!(resp.status(), 200);
    let id = resp
        .headers()
        .get("x-request-id")
        .expect("every response must carry x-request-id")
        .to_str()
        .unwrap();
    assert!(looks_like_uuid(id), "generated id should be a UUID: {id}");
}

#[tokio::test]
async fn test_request_id_propagated_from_client() {
    let base = spawn_full_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/get"))
        .header("x-request-id", "kong-correlation-abc")
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.headers().get("x-request-id").unwrap(),
        "kong-correlation-abc",
        "an inbound X-Request-Id must be propagated unchanged"
    );
}

#[tokio::test]
async fn test_request_id_present_on_404() {
    // Outermost placement means even unmatched routes get a correlation id.
    let base = spawn_full_app().await;
    let resp = reqwest::get(format!("{base}/no-such-route")).await.unwrap();

    assert_eq!(resp.status(), 404);
    assert!(
        resp.headers().get("x-request-id").is_some(),
        "404 responses must still carry x-request-id"
    );
}

#[tokio::test]
async fn test_request_id_does_not_clobber_handler_set_value() {
    // /response-headers lets a caller drive response headers; the request-id
    // middleware must not overwrite an x-request-id the handler set.
    let base = spawn_full_app().await;
    let resp = reqwest::get(format!("{base}/response-headers?x-request-id=handler-set"))
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers().get("x-request-id").unwrap(),
        "handler-set",
        "a handler-set x-request-id must win over the middleware"
    );
}

#[tokio::test]
async fn test_gzip_endpoint_forces_encoding() {
    use std::io::Read;
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/gzip")).await.unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CONTENT_ENCODING)
            .unwrap(),
        "gzip"
    );
    // The test reqwest client has no gzip decompression feature, so we get the
    // raw gzip bytes and gunzip them ourselves.
    let raw = resp.bytes().await.unwrap();
    let mut s = String::new();
    flate2::read::GzDecoder::new(&raw[..])
        .read_to_string(&mut s)
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(body["gzipped"], true);
}

#[tokio::test]
async fn test_cache_conditional_request() {
    let base = spawn_app().await;

    // No conditional header → 200 with validators.
    let resp = reqwest::get(format!("{base}/cache")).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.headers().get(reqwest::header::ETAG).is_some());
    assert!(resp.headers().get(reqwest::header::LAST_MODIFIED).is_some());

    // With If-None-Match → 304 Not Modified.
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/cache"))
        .header(reqwest::header::IF_NONE_MATCH, "\"rucho-cache-v1\"")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 304);
}

#[tokio::test]
async fn test_cache_seconds_sets_cache_control() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{base}/cache/120")).await.unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CACHE_CONTROL)
            .unwrap()
            .to_str()
            .unwrap(),
        "public, max-age=120"
    );
}

// --- TLS-info echo (real HTTPS via the TlsInfoAcceptor) ---

/// Spawns the REAL `build_app()` over HTTPS using `TlsInfoAcceptor` and a
/// committed self-signed fixture cert, returning the `https://127.0.0.1:PORT`
/// base URL. Exercises the same acceptor the binary's HTTPS listener uses, so
/// the negotiated TLS parameters genuinely flow through to the handlers.
async fn spawn_https_app() -> String {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let cert = format!("{manifest}/tests/fixtures/tls/cert.pem");
    let key = format!("{manifest}/tests/fixtures/tls/key.pem");

    let rustls_config =
        rucho::utils::server_config::try_load_rustls_config(Some(&cert), Some(&key))
            .await
            .expect("load self-signed TLS fixture");
    let acceptor = rucho::server::tls::TlsInfoAcceptor::new(rustls_config);

    let config = rucho::utils::config::Config::default();
    let metrics = Some(std::sync::Arc::new(rucho::utils::metrics::Metrics::new()));
    let chaos = std::sync::Arc::new(config.chaos.clone());
    let app = rucho::app::build_app(
        metrics,
        config.compression_enabled,
        chaos,
        config.max_body_size_bytes,
        config.request_id_enabled,
    );

    let handle = axum_server::Handle::new();
    let bind_handle = handle.clone();
    tokio::spawn(async move {
        axum_server::Server::bind("127.0.0.1:0".parse().unwrap())
            .acceptor(acceptor)
            .handle(bind_handle)
            .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
            .unwrap()
    });

    let addr = handle.listening().await.expect("HTTPS listener bound");
    format!("https://{addr}")
}

/// A reqwest client that trusts the self-signed fixture cert.
fn insecure_https_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
}

#[tokio::test]
async fn test_get_echoes_tls_info_over_https() {
    let base = spawn_https_app().await;
    let client = insecure_https_client();

    let resp = client.get(format!("{base}/get")).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    let tls = &body["tls"];
    assert!(tls.is_object(), "expected a tls object, got: {body}");
    assert!(
        tls["version"].as_str().unwrap_or("").starts_with("TLSv1."),
        "unexpected tls.version: {:?}",
        tls["version"]
    );
    assert!(
        tls["cipher_suite"].is_string(),
        "expected a cipher_suite string, got: {:?}",
        tls["cipher_suite"]
    );
    // ALPN is negotiated (reqwest offers h2/http1.1); allow null defensively.
    assert!(tls["alpn"].is_string() || tls["alpn"].is_null());
    // No mTLS configured → no client cert.
    assert_eq!(tls["client_cert_present"], false);
    assert_eq!(tls["client_certs"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_anything_echoes_tls_info_over_https() {
    let base = spawn_https_app().await;
    let client = insecure_https_client();

    let body: serde_json::Value = client
        .post(format!("{base}/anything"))
        .body("hello over tls")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(body["body"], "hello over tls");
    assert!(
        body["tls"]["version"]
            .as_str()
            .unwrap_or("")
            .starts_with("TLSv1."),
        "expected tls.version on /anything, got: {body}"
    );
}

#[tokio::test]
async fn test_get_omits_tls_info_over_plain_http() {
    // Over plain HTTP the extension is absent, so the `tls` key must not appear.
    let base = spawn_full_app().await;
    let body: serde_json::Value = reqwest::get(format!("{base}/get"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        body.get("tls").is_none(),
        "plain HTTP must omit tls, got: {body}"
    );
}
