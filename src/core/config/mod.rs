mod auto_tag;
mod cache_size;
mod change;
mod config_msg;
mod config_struct;
pub mod default;
mod json;
mod migrate;
mod song_pos_save;
mod version;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

use std::path::PathBuf;

pub use self::{
    auto_tag::*, cache_size::*, change::*, config_msg::*, config_struct::*,
    song_pos_save::*, version::*,
};

/// Unique app identifier, it is different when debugging.
#[cfg(not(debug_assertions))]
pub const APP_ID: &str = "uamp";
/// Unique app identifier, it is different when debugging.
#[cfg(debug_assertions)]
pub const APP_ID: &str = "uamp_debug";

/// Gets the default path for configuration, it is different when debugging.
pub fn default_config_dir() -> PathBuf {
    get_uamp_dir(dirs::config_dir())
}

/// Gets the default path for logs.
pub fn default_log_dir() -> PathBuf {
    let mut d = get_uamp_dir(dirs::data_local_dir());
    d.push("log");
    d
}

pub fn default_cache_dir() -> PathBuf {
    let mut d = get_uamp_dir(dirs::cache_dir());
    d.push("cache");
    d
}

pub fn default_http_client_path() -> PathBuf {
    "/usr/share/uamp/skins/default-uamp.tar".into()
}

pub fn default_plugin_folder() -> PathBuf {
    "/usr/lib/uamp/plugins".into()
}

/// Gets the default path to json configuration, it is different when debugging
pub fn default_config_path() -> PathBuf {
    default_config_dir().join("config.json")
}

pub const RELEASE_PORT: u16 = 8267;
pub const DEBUG_PORT: u16 = 33284;
#[cfg(not(debug_assertions))]
pub const DEFAULT_PORT: u16 = RELEASE_PORT;
#[cfg(debug_assertions)]
pub const DEFAULT_PORT: u16 = DEBUG_PORT;

fn get_uamp_dir(base: Option<PathBuf>) -> PathBuf {
    base.map(|mut a| {
        a.push(APP_ID);
        a
    })
    .unwrap_or_else(|| ".".into())
}
