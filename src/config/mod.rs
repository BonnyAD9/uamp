mod config_msg;
mod config_struct;
mod json;
mod song_pos_save;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

use self::json::*;
use std::path::PathBuf;

pub use self::{config_msg::*, config_struct::*};

/// Gets the unique app identifier, it is different when debugging.
pub fn _app_id() -> String {
    #[cfg(not(debug_assertions))]
    {
        "uamp".to_owned()
    }
    #[cfg(debug_assertions)]
    {
        "uamp_debug".to_owned()
    }
}

/// Gets the default path for configuration, it is different when debugging.
pub fn default_config_path() -> PathBuf {
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
