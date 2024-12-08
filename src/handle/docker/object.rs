use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct DockerConfig {
    #[serde(rename = "registry-mirrors")]
    pub(super) registry_mirrors: Vec<String>,
    #[serde(flatten)]
    extra_fields: HashMap<String, Value>,
}
