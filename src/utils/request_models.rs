use serde::Deserialize;
use utoipa::ToSchema;

/// Represents common query parameters for requests, such as `pretty`.
///
/// This struct is used to deserialize query parameters that control aspects like
/// the formatting of JSON responses.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PrettyQuery {
    /// If `true`, indicates that the JSON response should be pretty-printed.
    /// If `false` or absent, the JSON response will be compact.
    pub pretty: Option<bool>,
}
