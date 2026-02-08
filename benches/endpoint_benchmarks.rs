use criterion::{criterion_group, criterion_main, Criterion};

use axum::{body::Body, middleware, Router};
use http::Request;
use rucho::routes::{core_routes, delay, healthz};
use rucho::server::timing_layer::timing_middleware;
use tower::ServiceExt;

/// Builds a minimal app router for benchmarking.
///
/// Includes the core routes, healthz, and delay with timing middleware —
/// no chaos, metrics, tracing, or compression to isolate handler performance.
fn bench_app() -> Router {
    Router::new()
        .merge(core_routes::router())
        .merge(healthz::router())
        .merge(delay::router())
        .layer(middleware::from_fn(timing_middleware))
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

criterion_group!(
    benches,
    bench_get_healthz,
    bench_get_get,
    bench_get_uuid,
    bench_post_post,
    bench_get_endpoints,
);
criterion_main!(benches);
