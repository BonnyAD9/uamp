use std::sync::Arc;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct NewServer {
    address: Arc<String>,
    port: u16,
}

impl NewServer {
    pub fn new(port: u16, address: Arc<String>) -> Self {
        Self { port, address }
    }
}
