use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use toml::Value;

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct CargoConfig {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub(super) source: Source,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub(super) registries: Registryies,
    #[serde(flatten)]
    extra_fields: HashMap<String, Value>,
}

pub(super) type Source = HashMap<String, SourceValue>;
pub(super) type Registryies = HashMap<String, RegistryiesValue>;

pub(super) type SourceValue = HashMap<String, Value>;
pub(super) type RegistryiesValue = HashMap<String, Value>;
