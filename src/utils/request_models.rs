use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PrettyQuery {
    pub pretty: Option<bool>,
}
