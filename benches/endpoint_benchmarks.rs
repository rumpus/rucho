use criterion::{criterion_group, criterion_main, Criterion};

use axum::{body::Body, middleware, Router};
use http::Request;
use rucho::routes::{cookies, core_routes, delay, healthz, redirect};
use rucho::server::timing_layer::timing_middleware;
use std::sync::Arc;
use tower::ServiceExt;

/// Builds a minimal app router for benchmarking.
///
/// Includes the core routes, healthz, delay, redirect, and cookies with timing
/// middleware — no chaos, metrics, tracing, or compression to isolate handler
/// performance.
fn bench_app() -> Router {
    Router::new()
        .merge(core_routes::router())
        .merge(healthz::router())
        .merge(delay::router())
        .merge(redirect::router())
        .merge(cookies::router())
        .layer(middleware::from_fn(timing_middleware))
}

/// Builds the full application router exactly as the binary wires it (metrics,
/// chaos-gate, timing, trace, compression toggle, CORS, normalize-path,
/// request-id), for the bare-vs-full-stack comparison.
fn bench_full_app() -> Router {
    let config = rucho::utils::config::Config::default();
    rucho::app::build_app(
        Some(Arc::new(rucho::utils::metrics::Metrics::new())),
        config.compression_enabled,
        Arc::new(config.chaos.clone()),
        config.max_body_size_bytes,
        config.request_id_enabled,
    )
}

fn bench_get_healthz(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = bench_app();

    c.bench_function("GET /healthz", |b| {
        b.to_async(&rt).iter(|| {
            let app = app.clone();
            async move {
                let req = Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap();
                let resp = app.oneshot(req).await.unwrap();
                axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
            }
        });
    });
}

fn bench_get_get(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = bench_app();

    c.bench_function("GET /get", |b| {
        b.to_async(&rt).iter(|| {
            let app = app.clone();
            async move {
                let req = Request::builder()
                    .uri("/get")
                    .header("user-agent", "criterion-bench/1.0")
                    .header("accept", "application/json")
                    .body(Body::empty())
                    .unwrap();
                let resp = app.oneshot(req).await.unwrap();
                axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
            }
        });
    });
}

fn bench_get_uuid(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = bench_app();

    c.bench_function("GET /uuid", |b| {
        b.to_async(&rt).iter(|| {
            let app = app.clone();
            async move {
                let req = Request::builder().uri("/uuid").body(Body::empty()).unwrap();
                let resp = app.oneshot(req).await.unwrap();
                axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
            }
        });
    });
}

fn bench_post_post(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = bench_app();

    c.bench_function("POST /post", |b| {
        b.to_async(&rt).iter(|| {
            let app = app.clone();
            async move {
                let req = Request::builder()
                    .method("POST")
                    .uri("/post")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"key": "value"}"#))
                    .unwrap();
                let resp = app.oneshot(req).await.unwrap();
                axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
            }
        });
    });
}

fn bench_get_endpoints(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = bench_app();

    c.bench_function("GET /endpoints", |b| {
        b.to_async(&rt).iter(|| {
            let app = app.clone();
            async move {
                let req = Request::builder()
                    .uri("/endpoints")
                    .body(Body::empty())
                    .unwrap();
                let resp = app.oneshot(req).await.unwrap();
                axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
            }
        });
    });
}

fn bench_post_anything_with_body(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = bench_app();
    let body = r#"{"name":"rucho","values":[1,2,3,4,5],"nested":{"a":true,"b":"text"}}"#;

    c.bench_function("POST /anything (with body)", |b| {
        b.to_async(&rt).iter(|| {
            let app = app.clone();
            async move {
                let req = Request::builder()
                    .method("POST")
                    .uri("/anything")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap();
                let resp = app.oneshot(req).await.unwrap();
                axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
            }
        });
    });
}

fn bench_cookies_roundtrip(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = bench_app();

    c.bench_function("cookies roundtrip (set + read)", |b| {
        b.to_async(&rt).iter(|| {
            let app = app.clone();
            async move {
                // Set path: emits Set-Cookie headers + redirect.
                let set = Request::builder()
                    .uri("/cookies/set?session=abc123&theme=dark")
                    .body(Body::empty())
                    .unwrap();
                let r1 = app.clone().oneshot(set).await.unwrap();
                axum::body::to_bytes(r1.into_body(), usize::MAX)
                    .await
                    .unwrap();

                // Read path: parses the Cookie header back into JSON.
                let read = Request::builder()
                    .uri("/cookies")
                    .header("cookie", "session=abc123; theme=dark")
                    .body(Body::empty())
                    .unwrap();
                let r2 = app.oneshot(read).await.unwrap();
                axum::body::to_bytes(r2.into_body(), usize::MAX)
                    .await
                    .unwrap();
            }
        });
    });
}

fn bench_get_redirect(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = bench_app();

    c.bench_function("GET /redirect/3", |b| {
        b.to_async(&rt).iter(|| {
            let app = app.clone();
            async move {
                let req = Request::builder()
                    .uri("/redirect/3")
                    .body(Body::empty())
                    .unwrap();
                let resp = app.oneshot(req).await.unwrap();
                axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
            }
        });
    });
}

/// Compare a `GET /get` through the full middleware stack against the bare
/// handler (`bench_get_get`) to quantify the middleware overhead.
fn bench_get_get_full_stack(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = bench_full_app();

    c.bench_function("GET /get (full middleware stack)", |b| {
        b.to_async(&rt).iter(|| {
            let app = app.clone();
            async move {
                let req = Request::builder().uri("/get").body(Body::empty()).unwrap();
                let resp = app.oneshot(req).await.unwrap();
                axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
            }
        });
    });
}

/// Concurrent `record_request` to surface lock contention on the metrics map
/// (`RwLock<HashMap>`) — the baseline a future DashMap/sharded swap would beat.
fn bench_metrics_contention(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .build()
        .unwrap();
    let metrics = Arc::new(rucho::utils::metrics::Metrics::new());

    c.bench_function("Metrics::record_request (4 tasks x 100)", |b| {
        b.to_async(&rt).iter(|| {
            let metrics = metrics.clone();
            async move {
                let mut handles = Vec::with_capacity(4);
                for _ in 0..4 {
                    let m = metrics.clone();
                    handles.push(tokio::spawn(async move {
                        for _ in 0..100 {
                            m.record_request("/get", 200);
                        }
                    }));
                }
                for h in handles {
                    h.await.unwrap();
                }
            }
        });
    });
}

criterion_group!(
    benches,
    bench_get_healthz,
    bench_get_get,
    bench_get_uuid,
    bench_post_post,
    bench_get_endpoints,
    bench_post_anything_with_body,
    bench_cookies_roundtrip,
    bench_get_redirect,
    bench_get_get_full_stack,
    bench_metrics_contention,
);
criterion_main!(benches);
