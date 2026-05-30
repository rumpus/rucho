//! Forced content-encoding endpoints: `/gzip`, `/deflate`, `/brotli`.
//!
//! Each returns a JSON echo of the request (method + headers + a codec flag)
//! compressed with that codec and the matching `Content-Encoding` header —
//! REGARDLESS of the request's `Accept-Encoding`. Forcing the encoding is the
//! point: it gives a controllable upstream that emits an already-encoded body,
//! so you can observe how a gateway handles it (e.g. Kong's Response-Transformer
//! / RT-Advanced decode-and-rewrite path).
//!
//! This is distinct from the optional `CompressionLayer`, which only compresses
//! when the *client* negotiates it — and which correctly skips these responses
//! because they already carry a `Content-Encoding` (verified by a test below).

use std::io::Write;

use axum::{
    http::{header, HeaderMap, Method},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use flate2::{write::GzEncoder, write::ZlibEncoder, Compression};

use crate::routes::core_routes::serialize_headers;

/// Serializes the request-echo JSON (`{ "<flag>": true, "method", "headers" }`)
/// to bytes, ready to be compressed.
fn echo_json(codec_flag: &str, method: &Method, headers: &HeaderMap) -> Vec<u8> {
    let mut obj = serde_json::Map::new();
    obj.insert(codec_flag.to_owned(), serde_json::Value::Bool(true));
    obj.insert(
        "method".to_owned(),
        serde_json::Value::String(method.as_str().to_owned()),
    );
    obj.insert("headers".to_owned(), serialize_headers(headers));
    serde_json::to_vec(&serde_json::Value::Object(obj))
        .expect("infallible: serializing encoding echo body")
}

/// Builds the response: raw compressed `body`, `Content-Type: application/json`,
/// and the forced `Content-Encoding`.
fn encoded(content_encoding: &'static str, body: Vec<u8>) -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CONTENT_ENCODING, content_encoding),
        ],
        body,
    )
        .into_response()
}

/// Returns a gzip-encoded JSON echo of the request (`Content-Encoding: gzip`).
#[utoipa::path(
    get,
    path = "/gzip",
    responses((status = 200, description = "gzip-encoded JSON echo of the request"))
)]
pub async fn gzip_handler(method: Method, headers: HeaderMap) -> Response {
    let json = echo_json("gzipped", &method, &headers);
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(&json).expect("infallible: gzip write to Vec");
    encoded("gzip", enc.finish().expect("infallible: gzip finish"))
}

/// Returns a deflate-encoded JSON echo (`Content-Encoding: deflate`).
///
/// Uses the zlib container (matching httpbin's `zlib.compress`), which is what
/// `Content-Encoding: deflate` means in practice for real-world clients.
#[utoipa::path(
    get,
    path = "/deflate",
    responses((status = 200, description = "deflate-encoded JSON echo of the request"))
)]
pub async fn deflate_handler(method: Method, headers: HeaderMap) -> Response {
    let json = echo_json("deflated", &method, &headers);
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    enc.write_all(&json)
        .expect("infallible: deflate write to Vec");
    encoded("deflate", enc.finish().expect("infallible: deflate finish"))
}

/// Returns a brotli-encoded JSON echo (`Content-Encoding: br`).
#[utoipa::path(
    get,
    path = "/brotli",
    responses((status = 200, description = "brotli-encoded JSON echo of the request"))
)]
pub async fn brotli_handler(method: Method, headers: HeaderMap) -> Response {
    let json = echo_json("brotli", &method, &headers);
    let mut compressed = Vec::new();
    let mut input = json.as_slice();
    let params = brotli::enc::BrotliEncoderParams::default();
    brotli::BrotliCompress(&mut input, &mut compressed, &params)
        .expect("infallible: brotli compress to Vec");
    encoded("br", compressed)
}

/// Creates and returns the Axum router for the forced-encoding endpoints.
pub fn router() -> Router {
    Router::new()
        .route("/gzip", get(gzip_handler))
        .route("/deflate", get(deflate_handler))
        .route("/brotli", get(brotli_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::io::Read;
    use tower::ServiceExt;

    async fn fetch(path: &str) -> Response {
        router()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn body_bytes(resp: Response) -> Vec<u8> {
        axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec()
    }

    #[tokio::test]
    async fn test_gzip_forced_and_decodes() {
        let resp = fetch("/gzip").await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_ENCODING).unwrap(),
            "gzip"
        );
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
        let compressed = body_bytes(resp).await;
        let mut s = String::new();
        flate2::read::GzDecoder::new(&compressed[..])
            .read_to_string(&mut s)
            .unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["gzipped"], true);
        assert_eq!(v["method"], "GET");
    }

    #[tokio::test]
    async fn test_deflate_forced_and_decodes() {
        let resp = fetch("/deflate").await;
        assert_eq!(
            resp.headers().get(header::CONTENT_ENCODING).unwrap(),
            "deflate"
        );
        let compressed = body_bytes(resp).await;
        let mut s = String::new();
        flate2::read::ZlibDecoder::new(&compressed[..])
            .read_to_string(&mut s)
            .unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["deflated"], true);
    }

    #[tokio::test]
    async fn test_brotli_forced_and_decodes() {
        let resp = fetch("/brotli").await;
        assert_eq!(resp.headers().get(header::CONTENT_ENCODING).unwrap(), "br");
        let compressed = body_bytes(resp).await;
        let mut decompressed = Vec::new();
        let mut input = compressed.as_slice();
        brotli::BrotliDecompress(&mut input, &mut decompressed).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&decompressed).unwrap();
        assert_eq!(v["brotli"], true);
    }

    #[tokio::test]
    async fn test_compression_layer_does_not_double_encode() {
        use tower_http::compression::CompressionLayer;
        // Even with the client negotiating gzip AND the CompressionLayer active,
        // the already-gzip-encoded /gzip body must not be re-compressed.
        let app = router().layer(CompressionLayer::new());
        let resp = app
            .oneshot(
                Request::get("/gzip")
                    .header(header::ACCEPT_ENCODING, "gzip")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            resp.headers().get(header::CONTENT_ENCODING).unwrap(),
            "gzip"
        );
        let compressed = body_bytes(resp).await;
        // A single gunzip must yield valid JSON — double-encoding would fail here.
        let mut s = String::new();
        flate2::read::GzDecoder::new(&compressed[..])
            .read_to_string(&mut s)
            .unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&s).is_ok());
    }
}
