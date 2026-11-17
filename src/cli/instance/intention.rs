use std::path::PathBuf;

use crate::core::utils::exim::EximFormat;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum Intention {
    #[default]
    Default,
    Clear,
    Export(PathBuf, Option<EximFormat>),
}
