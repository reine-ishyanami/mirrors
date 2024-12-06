use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use toml::Value;

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct PipConfig {
    pub(super) global: Global,
    pub(super) install: Install,
    #[serde(flatten)]
    extra_fields: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct Global {
    #[serde(rename = "index-dir", skip_serializing_if = "String::is_empty")]
    pub(super) index_url: String,
    #[serde(flatten)]
    extra_fields: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct Install {
    #[serde(rename = "trusted-host", skip_serializing_if = "String::is_empty")]
    pub(super) trusted_host: String,
    #[serde(flatten)]
    extra_fields: HashMap<String, Value>,
}
