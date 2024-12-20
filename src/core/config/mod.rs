mod config_msg;
mod config_struct;
mod json;
mod song_pos_save;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

use std::path::PathBuf;

pub use self::{config_msg::*, config_struct::*};

/// Unique app identifier, it is different when debugging.
#[cfg(not(debug_assertions))]
pub const APP_ID: &str = "uamp";
/// Unique app identifier, it is different when debugging.
#[cfg(debug_assertions)]
pub const APP_ID: &str = "uamp_debug";

/// Version of uamp as string.
pub const VERSION_STR: &str = {
    let v = option_env!("CARGO_PKG_VERSION");
    if let Some(v) = v {
        v
    } else {
        "unknown"
    }
};

/// Gets the default path for configuration, it is different when debugging.
pub fn default_config_dir() -> PathBuf {
    if let Some(dir) = dirs::config_dir() {
        // use different path when debugging to not ruin existing config
        #[cfg(not(debug_assertions))]
        {
            dir.join("uamp")
        }
        #[cfg(debug_assertions)]
        {
            dir.join("uamp_debug")
        }
    } else {
        PathBuf::from(".")
    }
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
