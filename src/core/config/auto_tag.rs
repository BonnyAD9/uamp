use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTag {
    pub name: Arc<str>,
    #[serde(default = "default_hidden")]
    pub hidden: bool,
}

impl AutoTag {
    pub fn new(name: impl Into<Arc<str>>, hidden: bool) -> Self {
        Self {
            name: name.into(),
            hidden,
        }
    }

    pub fn hidden(name: impl Into<Arc<str>>) -> Self {
        Self::new(name, true)
    }

    pub fn visible(name: impl Into<Arc<str>>) -> Self {
        Self::new(name, false)
    }
}

fn default_hidden() -> bool {
    true
}
