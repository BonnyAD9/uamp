use serde::Serialize;

use crate::core::{config::VERSION_STR, messenger};

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct Info<'a> {
    version: &'static str,
    info: &'a messenger::Info,
}

impl<'a> Info<'a> {
    pub fn new(info: &'a messenger::Info) -> Self {
        Self {
            info,
            version: VERSION_STR,
        }
    }
}
