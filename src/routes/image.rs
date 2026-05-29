//! Image endpoint — returns a small sample image in a requested format.
//!
//! `/image/:format` (png, jpeg, svg, webp) returns a tiny fixed sample image
//! with the matching `Content-Type`. A controllable upstream for testing how a
//! gateway handles binary/image responses: content-type routing, image-specific
//! plugins, and compression decisions (a gateway should generally NOT re-compress
//! already-compressed raster formats, but may compress the text-based SVG).
//!
//! The raster fixtures (`assets/sample.{png,jpeg,webp}`) are embedded at compile
//! time via `include_bytes!`; SVG is an inline string. Bodies are served as
//! `&'static` slices — no per-request allocation.

use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use crate::utils::error_response::format_error_response;

/// 16x16 sample PNG (steel-blue with a white border + diagonal).
const SAMPLE_PNG: &[u8] = include_bytes!("assets/sample.png");
/// 16x16 sample JPEG (same image).
const SAMPLE_JPEG: &[u8] = include_bytes!("assets/sample.jpeg");
/// 16x16 sample WebP (same image).
const SAMPLE_WEBP: &[u8] = include_bytes!("assets/sample.webp");
/// 16x16 sample SVG (vector equivalent — text, so a gateway may compress it).
const SAMPLE_SVG: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
  <rect width="16" height="16" fill="#4682b4" stroke="#ffffff"/>
  <line x1="0" y1="0" x2="16" y2="16" stroke="#ffffff"/>
</svg>
"##;

/// Returns a small sample image in the requested `format`.
///
/// Supported: `png`, `jpeg` (alias `jpg`), `svg`, `webp` — any other value
/// returns 400. Each returns a tiny fixed image with the matching
/// `Content-Type`, as a controllable upstream for testing gateway handling of
/// binary/image bodies.
#[utoipa::path(
    get,
    path = "/image/{format}",
    params(
        ("format" = String, Path, description = "Image format: png, jpeg, svg, or webp")
    ),
    responses(
        (status = 200, description = "A sample image in the requested format"),
        (status = 400, description = "Unsupported image format")
    )
)]
pub async fn image_handler(Path(format): Path<String>) -> Response {
    let (content_type, body): (&str, &'static [u8]) = match format.to_ascii_lowercase().as_str() {
        "png" => ("image/png", SAMPLE_PNG),
        "jpeg" | "jpg" => ("image/jpeg", SAMPLE_JPEG),
        "webp" => ("image/webp", SAMPLE_WEBP),
        "svg" => ("image/svg+xml", SAMPLE_SVG.as_bytes()),
        other => {
            return format_error_response(
                StatusCode::BAD_REQUEST,
                &format!("Unsupported image format '{other}'. Supported: png, jpeg, svg, webp"),
            );
        }
    };

    ([(header::CONTENT_TYPE, content_type)], body).into_response()
}

/// Creates and returns the Axum router for the image endpoint.
pub fn router() -> Router {
    Router::new().route("/image/:format", get(image_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn fetch(format: &str) -> Response {
        router()
            .oneshot(
                Request::get(format!("/image/{format}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_png_content_type_and_magic() {
        let response = fetch("png").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/png"
        );
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        // PNG magic number.
        assert_eq!(&body[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[tokio::test]
    async fn test_jpeg_content_type_and_magic() {
        let response = fetch("jpeg").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/jpeg"
        );
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        // JPEG SOI marker.
        assert_eq!(&body[..2], b"\xff\xd8");
    }

    #[tokio::test]
    async fn test_webp_content_type_and_magic() {
        let response = fetch("webp").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/webp"
        );
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        // RIFF container + WEBP fourcc.
        assert_eq!(&body[..4], b"RIFF");
        assert_eq!(&body[8..12], b"WEBP");
    }

    #[tokio::test]
    async fn test_svg_content_type_and_body() {
        let response = fetch("svg").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/svg+xml"
        );
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("<svg"));
    }

    #[tokio::test]
    async fn test_jpg_alias() {
        let response = fetch("jpg").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/jpeg"
        );
    }

    #[tokio::test]
    async fn test_unsupported_format_returns_400() {
        let response = fetch("gif").await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_format_is_case_insensitive() {
        let response = fetch("PNG").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/png"
        );
    }
}
