use serde::Serialize;

use crate::core::{config::VERSION_STR, server};

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct Info<'a> {
    version: &'static str,
    info: &'a server::Info,
}

impl<'a> Info<'a> {
    pub fn new(info: &'a server::Info) -> Self {
        Self {
            info,
            version: VERSION_STR,
        }
    }
}
