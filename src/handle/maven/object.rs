use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Mirrors {
    pub(super) mirror: Vec<Mirror>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Mirror {
    pub(super) id: String,
    pub(super) name: String,
    #[serde(rename = "mirrorOf")]
    pub(super) mirror_of: String,
    pub(super) url: String,
}
