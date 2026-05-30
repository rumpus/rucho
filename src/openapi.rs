//! OpenAPI specification for the Rucho API.
//!
//! `ApiDoc` aggregates every documented path and schema into the spec served at
//! `/api-docs/openapi.json` and rendered by Swagger UI. It lives in the library
//! (not the binary) so integration tests can assert the spec shape.

use utoipa::OpenApi;

use crate::routes::core_routes::EndpointInfo;

/// OpenAPI documentation aggregator for the Rucho API.
///
/// Used by `utoipa` to generate the OpenAPI specification; it aggregates all
/// the paths and components (schemas, responses, etc.) that make up the API.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::core_routes::root_handler,
        crate::routes::core_routes::get_handler,
        crate::routes::core_routes::head_handler,
        crate::routes::core_routes::post_handler,
        crate::routes::core_routes::put_handler,
        crate::routes::core_routes::patch_handler,
        crate::routes::core_routes::delete_handler,
        crate::routes::core_routes::options_handler,
        crate::routes::core_routes::status_handler,
        crate::routes::core_routes::anything_handler,
        crate::routes::core_routes::anything_path_handler,
        crate::routes::core_routes::endpoints_handler,
        crate::routes::delay::delay_handler,
        crate::routes::healthz::healthz_handler,
        crate::routes::redirect::redirect_handler,
        crate::routes::cookies::cookies_handler,
        crate::routes::cookies::set_cookies_handler,
        crate::routes::cookies::delete_cookies_handler,
        crate::routes::base64::base64_handler,
        crate::routes::bytes::bytes_handler,
        crate::routes::drip::drip_handler,
        crate::routes::encoding::gzip_handler,
        crate::routes::encoding::deflate_handler,
        crate::routes::encoding::brotli_handler,
        crate::routes::response_headers::response_headers_handler,
        crate::routes::content_types::xml_handler,
        crate::routes::content_types::html_handler,
        crate::routes::image::image_handler,
        crate::routes::range::range_handler,
        crate::routes::core_routes::uuid_handler,
        crate::routes::core_routes::ip_handler,
        crate::routes::core_routes::user_agent_handler,
        crate::routes::core_routes::headers_handler,
    ),
    components(
        schemas(EndpointInfo, crate::routes::core_routes::Payload)
    ),
    tags(
        (name = "Rucho", description = "Rucho API")
    )
)]
pub struct ApiDoc;
