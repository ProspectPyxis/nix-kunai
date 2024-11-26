use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct Source {
    pub version: String,
    pub hash: String,
    pub tag_prefix_filter: Option<String>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct SourceMap {
    #[serde(flatten)]
    pub sources: HashMap<String, Source>,
}

impl Source {
    pub fn new(version: &str) -> Self {
        Source {
            version: version.to_string(),
            hash: "".to_string(),
            tag_prefix_filter: None,
        }
    }
}
