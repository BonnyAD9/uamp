use crate::core::config::{self, ConfigClone};

pub type Config = ConfigClone;

impl Config {
    pub fn new(c: &config::Config) -> Self {
        c.partial_clone()
    }
}
