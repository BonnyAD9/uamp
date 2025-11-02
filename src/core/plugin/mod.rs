mod ctypes;
mod decoder_plugin;

use std::{
    ffi::{CStr, c_char},
    path::Path,
};

use libloading::Symbol;

use crate::core::{Error, Result};

pub use self::decoder_plugin::*;

const CURRENT_VERSION: u32 = 0x00_001_000;

#[derive(Debug)]
pub enum Plugin {
    Decoder(DecoderPlugin),
}

#[repr(i32)]
#[derive(Debug)]
enum PluginType {
    Decoder = 1,
}

#[repr(C)]
#[derive(Debug)]
struct PluginConfig {
    version: u32,
    name: *const c_char,
    typ: i32,
}

impl Plugin {
    pub fn load(p: impl AsRef<Path>) -> Result<Plugin> {
        unsafe {
            let lib = libloading::Library::new(p.as_ref())?;
            let cfg: Symbol<*const PluginConfig> =
                lib.get(b"uamp_plugin_config\0")?;
            if (**cfg).version != CURRENT_VERSION {
                return Error::invalid_value()
                    .msg("Invalid plugin version.")
                    .err();
            }
            let Some(t) = PluginType::from_id((**cfg).typ) else {
                return Error::invalid_value()
                    .msg("Unknown plugin type.")
                    .err();
            };
            let name =
                CStr::from_ptr((**cfg).name).to_string_lossy().into_owned();

            match t {
                PluginType::Decoder => {
                    Ok(Plugin::Decoder(DecoderPlugin::load(lib, name)?))
                }
            }
        }
    }
}

impl PluginType {
    fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::Decoder),
            _ => None,
        }
    }
}
