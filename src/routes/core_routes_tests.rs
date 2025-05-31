// src/routes/core_routes_tests.rs

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header::{HeaderValue, CONTENT_TYPE}},
    };
    use tower::ServiceExt; // for `app.oneshot()`
    use serde_json::{json, Value};
    use crate::routes::core_routes; // Assuming core_routes::router() is the target
    use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
    use base64::Engine;

    // Helper function to create the app router for testing
    fn app() -> Router {
        core_routes::router()
        // In a real scenario, you might need to merge other routers if they affect /anything
        // For this handler, core_routes::router() should be sufficient.
    }

    #[tokio::test]
    async fn test_anything_text_plain() {
        let app = app();
        let request_body = "Hello, world!";

        let request = Request::builder()
            .method("POST")
            .uri("/anything")
            .header(CONTENT_TYPE, HeaderValue::from_static("text/plain"))
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["method"], "POST");
        assert_eq!(body_json["path"], "/anything");
        assert_eq!(body_json["detected_content_type"], "text/plain");
        assert_eq!(body_json["parsed_body"], request_body);
    }

    #[tokio::test]
    async fn test_anything_application_json() {
        let app = app();
        let request_json = json!({"key": "value", "number": 123});
        let request_body_str = request_json.to_string();

        let request = Request::builder()
            .method("POST")
            .uri("/anything")
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(Body::from(request_body_str))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["detected_content_type"], "application/json");
        assert_eq!(body_json["parsed_body"], request_json);
    }

    #[tokio::test]
    async fn test_anything_malformed_json() {
        let app = app();
        let request_body_str = "{ \"key\": \"value\", "; // Intentionally malformed

        let request = Request::builder()
            .method("POST")
            .uri("/anything")
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(Body::from(request_body_str.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["detected_content_type"], "application/json");
        assert!(body_json["parsed_body"]["error"].as_str().is_some());
        assert_eq!(body_json["parsed_body"]["error"], "Failed to parse JSON body");
        assert_eq!(body_json["parsed_body"]["original_body_utf8_lossy"], request_body_str);
    }

    #[tokio::test]
    async fn test_anything_x_www_form_urlencoded() {
        let app = app();
        let request_body_str = "name=test&project=Rucho";
        let expected_json = json!({"name": "test", "project": "Rucho"});

        let request = Request::builder()
            .method("POST")
            .uri("/anything")
            .header(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"))
            .body(Body::from(request_body_str.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["detected_content_type"], "application/x-www-form-urlencoded");
        assert_eq!(body_json["parsed_body"], expected_json);
    }

    #[tokio::test]
    async fn test_anything_malformed_x_www_form_urlencoded() {
        let app = app();
        let request_body_str = "name=test%&project=Rucho"; // malformed %

        let request = Request::builder()
            .method("POST")
            .uri("/anything")
            .header(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"))
            .body(Body::from(request_body_str.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["detected_content_type"], "application/x-www-form-urlencoded");
        assert!(body_json["parsed_body"]["error"].as_str().is_some());
        assert_eq!(body_json["parsed_body"]["error"], "Failed to parse x-www-form-urlencoded body");
        assert_eq!(body_json["parsed_body"]["original_body_utf8_lossy"], request_body_str);
    }

    #[tokio::test]
    async fn test_anything_multipart_form_data() {
        let app = app();
        let request_body_str = "--boundary\r\nContent-Disposition: form-data; name=\"field1\"\r\n\r\nvalue1\r\n--boundary--\r\n";
        let content_type_val = "multipart/form-data; boundary=boundary";

        let request = Request::builder()
            .method("POST")
            .uri("/anything")
            .header(CONTENT_TYPE, HeaderValue::from_str(content_type_val).unwrap())
            .body(Body::from(request_body_str.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["detected_content_type"], content_type_val);
        assert!(body_json["parsed_body"]["message"].as_str().is_some());
        assert_eq!(body_json["parsed_body"]["message"], "Multipart content detected. Full parsing in /anything is complex due to pre-consumed body. Body shown as Base64.");
        assert_eq!(body_json["parsed_body"]["original_body_base64"], BASE64_STANDARD.encode(request_body_str.as_bytes()));
    }

    #[tokio::test]
    async fn test_anything_application_octet_stream() {
        let app = app();
        let request_body_bytes = vec![1, 2, 3, 4, 5];
        let expected_base64 = BASE64_STANDARD.encode(&request_body_bytes);

        let request = Request::builder()
            .method("POST")
            .uri("/anything")
            .header(CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"))
            .body(Body::from(request_body_bytes.clone()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes_resp = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes_resp).unwrap();

        assert_eq!(body_json["detected_content_type"], "application/octet-stream");
        assert_eq!(body_json["parsed_body"], expected_base64);
    }

    #[tokio::test]
    async fn test_anything_text_html() {
        let app = app();
        let request_body = "<html><body><h1>Hello</h1></body></html>";

        let request = Request::builder()
            .method("POST")
            .uri("/anything")
            .header(CONTENT_TYPE, HeaderValue::from_static("text/html"))
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["detected_content_type"], "text/html");
        assert_eq!(body_json["parsed_body"], request_body);
    }

    #[tokio::test]
    async fn test_anything_application_xml() {
        let app = app();
        let request_body = "<root><item>Test</item></root>";

        let request = Request::builder()
            .method("POST")
            .uri("/anything")
            .header(CONTENT_TYPE, HeaderValue::from_static("application/xml"))
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["detected_content_type"], "application/xml");
        assert_eq!(body_json["parsed_body"], request_body);
    }

    #[tokio::test]
    async fn test_anything_no_content_type() {
        let app = app();
        let request_body_str = "some data";
        // Default content type in handler is "application/octet-stream"
        // which then gets base64 encoded if not valid UTF-8, or shown as string if it is.
        // "some data" is valid UTF-8.
        // The logic in anything_handler is:
        // "application/octet-stream" | _ => {
        //   if content_type_header == "application/octet-stream" { // This will be true
        //        json!(base64::encode(&body_bytes))
        //   } else { ... }
        // So, it should be base64 encoded.

        let request = Request::builder()
            .method("POST")
            .uri("/anything")
            // No Content-Type header
            .body(Body::from(request_body_str.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes_resp = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes_resp).unwrap();

        assert_eq!(body_json["detected_content_type"], "application/octet-stream"); // Default
        assert_eq!(body_json["parsed_body"], BASE64_STANDARD.encode(request_body_str.as_bytes()));
    }

    #[tokio::test]
    async fn test_anything_unsupported_content_type() {
        let app = app();
        let request_body_str = "some data for unsupported type";
        // The logic for unknown types (not "application/octet-stream") is:
        // match String::from_utf8(body_bytes.to_vec()) {
        //     Ok(s) => json!(s),
        //     Err(_) => json!({ "message": "Body is not valid UTF-8, showing as Base64", "body_base64": base64::encode(&body_bytes) })
        // }
        // "some data for unsupported type" is valid UTF-8.

        let request = Request::builder()
            .method("POST")
            .uri("/anything")
            .header(CONTENT_TYPE, HeaderValue::from_static("application/unsupported-type"))
            .body(Body::from(request_body_str.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes_resp = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes_resp).unwrap();

        assert_eq!(body_json["detected_content_type"], "application/unsupported-type");
        assert_eq!(body_json["parsed_body"], request_body_str); // Should be parsed as UTF-8 string
    }
}
