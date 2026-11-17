use std::path::PathBuf;

use pareg::FromArgs;

use crate::core::{query::Query, utils::exim::EximFormat};

#[derive(Debug, FromArgs)]
#[from_args(match start {
    "--" => break,
    "--m3u" => format = Some(EximFormat::M3u),
    "--m3u8" => format = Some(EximFormat::M3u8),
    "--pls" => format = Some(EximFormat::Pls),
})]
#[from_args(positional_guard)]
pub struct ExportPlaylist {
    #[from_args("--output", "-o", positional)]
    pub path: PathBuf,
    #[from_args("--query", "-q", positional, default = Query::current_top())]
    pub query: Query,
    #[from_args("--format", option)]
    pub format: Option<EximFormat>,
}
