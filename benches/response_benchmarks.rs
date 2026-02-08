use criterion::{black_box, criterion_group, criterion_main, Criterion};

use axum::http::StatusCode;
use rucho::utils::error_response::format_error_response;
use rucho::utils::json_response::{format_json_response, format_json_response_with_timing};
use serde_json::json;

fn bench_format_json_response_small(c: &mut Criterion) {
    c.bench_function("format_json_response (small)", |b| {
        b.iter(|| {
            let data = json!({"message": "hello"});
            black_box(format_json_response(data));
        });
    });
}

fn bench_format_json_response_medium(c: &mut Criterion) {
    c.bench_function("format_json_response (medium)", |b| {
        b.iter(|| {
            let data = json!({
                "method": "GET",
                "path": "/get",
                "query": "",
                "origin": "127.0.0.1",
                "url": "http://localhost:8080/get",
                "headers": {
                    "host": "localhost:8080",
                    "user-agent": "curl/7.81.0",
                    "accept": "*/*",
                    "x-forwarded-for": "192.168.1.100",
                    "x-request-id": "abc-123-def-456"
                },
                "body": ""
            });
            black_box(format_json_response(data));
        });
    });
}

fn bench_format_json_response_with_timing(c: &mut Criterion) {
    c.bench_function("format_json_response_with_timing (medium)", |b| {
        b.iter(|| {
            let data = json!({
                "method": "GET",
                "path": "/get",
                "query": "",
                "origin": "127.0.0.1",
                "url": "http://localhost:8080/get",
                "headers": {
                    "host": "localhost:8080",
                    "user-agent": "curl/7.81.0",
                    "accept": "*/*",
                    "x-forwarded-for": "192.168.1.100",
                    "x-request-id": "abc-123-def-456"
                },
                "body": ""
            });
            black_box(format_json_response_with_timing(data, Some(0.42)));
        });
    });
}

fn bench_format_error_response(c: &mut Criterion) {
    c.bench_function("format_error_response", |b| {
        b.iter(|| {
            black_box(format_error_response(StatusCode::NOT_FOUND, "Not found"));
        });
    });
}

criterion_group!(
    benches,
    bench_format_json_response_small,
    bench_format_json_response_medium,
    bench_format_json_response_with_timing,
    bench_format_error_response,
);
criterion_main!(benches);
