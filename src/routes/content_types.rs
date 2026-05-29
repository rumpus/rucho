//! Static document endpoints — return non-JSON content types.
//!
//! `/xml` and `/html` emit small, valid sample documents with the matching
//! `Content-Type`. They deliberately break Rucho's JSON-everywhere convention
//! (like `/bytes`): the point is a controllable upstream that returns non-JSON
//! bodies, for exercising gateway behavior that varies by content type —
//! response transformers, content-type routing, and compression decisions
//! (text compresses, so a gateway may gzip these where it skips `/bytes`).

use axum::{
    http::header,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

/// A small, valid sample XML document returned by `/xml`.
const SAMPLE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rucho>
  <message>This is a sample XML document served by Rucho.</message>
  <purpose>Test how a gateway handles non-JSON (application/xml) upstream responses.</purpose>
</rucho>
"#;

/// A small, valid sample HTML document returned by `/html`.
const SAMPLE_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Rucho</title>
</head>
<body>
  <h1>Rucho</h1>
  <p>This is a sample HTML document served by Rucho.</p>
  <p>Use it to test how a gateway handles non-JSON (text/html) upstream responses.</p>
</body>
</html>
"#;

/// Returns a small sample XML document as `application/xml`.
///
/// Deliberately non-JSON — a controllable upstream for testing how a gateway
/// handles `application/xml` responses (e.g. content-type-aware plugins).
#[utoipa::path(
    get,
    path = "/xml",
    responses(
        (status = 200, description = "A sample XML document", content_type = "application/xml", body = String)
    )
)]
pub async fn xml_handler() -> Response {
    ([(header::CONTENT_TYPE, "application/xml")], SAMPLE_XML).into_response()
}

/// Returns a small sample HTML document as `text/html; charset=utf-8`.
///
/// Deliberately non-JSON — a controllable upstream for testing how a gateway
/// handles `text/html` responses.
#[utoipa::path(
    get,
    path = "/html",
    responses(
        (status = 200, description = "A sample HTML document", content_type = "text/html", body = String)
    )
)]
pub async fn html_handler() -> Response {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        SAMPLE_HTML,
    )
        .into_response()
}

/// Creates and returns the Axum router for the content-type document endpoints.
pub fn router() -> Router {
    Router::new()
        .route("/xml", get(xml_handler))
        .route("/html", get(html_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_xml_returns_application_xml() {
        let app = router();
        let response = app
            .oneshot(Request::get("/xml").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/xml"
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.starts_with("<?xml"));
        assert!(text.contains("<rucho>"));
    }

    #[tokio::test]
    async fn test_html_returns_text_html() {
        let app = router();
        let response = app
            .oneshot(Request::get("/html").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("<!DOCTYPE html>"));
        assert!(text.contains("<h1>Rucho</h1>"));
    }
}
