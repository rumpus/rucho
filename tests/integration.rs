//! Integration tests for Rucho HTTP endpoints.
//!
//! Each test spins up a real HTTP server on a random port and hits it with
//! `reqwest`, validating status codes, headers, and response bodies over
//! actual TCP connections.

use axum::{middleware, Router};
use rucho::routes::{cookies, core_routes, delay, healthz, redirect};
use rucho::server::timing_layer::timing_middleware;

/// Spawns a test server on a random port and returns its base URL.
///
/// Builds a minimal Router (no chaos, metrics, tracing, or swagger) and
/// serves it on `127.0.0.1:0` so each test gets its own isolated server.
async fn spawn_app() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let app = Router::new()
        .merge(core_routes::router())
        .merge(healthz::router())
        .merge(delay::router())
        .merge(redirect::router())
        .merge(cookies::router())
        .layer(middleware::from_fn(timing_middleware));

    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

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
    assert!(body["origin"].is_string());
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
