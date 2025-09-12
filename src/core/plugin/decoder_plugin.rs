use std::{
    ffi::{c_char, c_void},
    path::Path,
    sync::Arc,
    time::Duration,
};

use anyhow::anyhow;
use bitflags::bitflags;
use libloading::Library;
use log::warn;
use raplay::{
    Callback, SampleBufferMut, Source, Timestamp,
    source::{DeviceConfig, VolumeIterator},
};

const CURRENT_VERSION: u32 = 0x00_001_000;
const MAX_ERRS: usize = 1000;

use crate::core::{
    Error, Result,
    plugin::ctypes::{
        CDeviceConfig, CDuration, CError, CErrorType, CSampleFormat,
        CTimestamp, CVolumeIterator, OpaqueType,
    },
};

#[derive(Debug)]
pub struct DecoderPlugin {
    open: unsafe extern "C" fn(*const c_char, usize) -> *mut c_void,
    free: unsafe extern "C" fn(*mut c_void),

    // MUST BE LAST to drop
    imp: Arc<DecoderPluginSourceImpl>,
}

#[derive(Debug)]
struct DecoderPluginSourceImpl {
    name: String,

    err: unsafe extern "C" fn(*mut c_void) -> CError,
    set_volume: Option<unsafe extern "C" fn(*mut c_void, CVolumeIterator)>,
    set_config: unsafe extern "C" fn(*mut c_void, *const CDeviceConfig),
    read: unsafe extern "C" fn(
        *mut c_void,
        *mut c_void,
        usize,
        CSampleFormat,
    ) -> usize,
    preferred_config:
        Option<unsafe extern "C" fn(*mut c_void) -> CDeviceConfig>,
    seek: Option<
        unsafe extern "C" fn(*mut c_void, time: CDuration) -> CTimestamp,
    >,
    seek_by: Option<
        unsafe extern "C" fn(
            *mut c_void,
            time: CDuration,
            forward: bool,
        ) -> CTimestamp,
    >,
    get_time: Option<unsafe extern "C" fn(*const c_void) -> CTimestamp>,

    // MUST BE LAST in the drop order
    _lib: Library,
}

impl DecoderPlugin {
    pub fn load(lib: Library, name: String) -> Result<Self> {
        unsafe {
            let cfg: *const DecoderPluginConfig =
                *lib.get(b"uamp_plugin_decoder_config\0")?;
            if (*cfg).version != CURRENT_VERSION {
                return Err(Error::invalid_value(
                    "Unknown decoder plugin version.",
                ));
            }

            let flags = &(*cfg).flags;

            let open = *lib.get(b"uamp_decoder_open\0")?;
            let err = *lib.get(b"uamp_decoder_err\0")?;
            let free = *lib.get(b"uamp_decoder_free\0")?;
            let set_volume = if flags.contains(DecoderPluginFlags::VOLUME) {
                Some(*lib.get(b"uamp_decoder_set_volume\0")?)
            } else {
                None
            };
            let set_config = *lib.get(b"uamp_decoder_set_config\0")?;
            let read = *lib.get(b"uamp_decoder_read\0")?;
            let preferred_config =
                if flags.contains(DecoderPluginFlags::CONFIG) {
                    Some(*lib.get(b"uamp_decoder_preferred_config\0")?)
                } else {
                    None
                };
            let seek = if flags.contains(DecoderPluginFlags::SEEK) {
                Some(*lib.get(b"uamp_decoder_seek\0")?)
            } else {
                None
            };
            let seek_by = if flags.contains(DecoderPluginFlags::SEEK_BY) {
                Some(*lib.get(b"uamp_decoder_seek_by\0")?)
            } else {
                None
            };
            let get_time = if flags.contains(DecoderPluginFlags::GET_TIME) {
                Some(*lib.get(b"uamp_decoder_get_time\0")?)
            } else {
                None
            };

            let imp = Arc::new(DecoderPluginSourceImpl {
                name,
                err,
                set_volume,
                set_config,
                read,
                preferred_config,
                seek,
                seek_by,
                get_time,

                _lib: lib,
            });

            Ok(Self { open, free, imp })
        }
    }

    pub fn open(&self, p: impl AsRef<Path>) -> Result<Box<dyn Source>> {
        let path = p.as_ref().as_os_str().as_encoded_bytes();
        let data = unsafe {
            let inst = (self.open)(path.as_ptr() as *const _, path.len());
            let data = OpaqueType::new(inst, self.free);
            get_errors(inst, self.imp.err, &self.imp.name)?;
            data
        };

        Ok(Box::new(DecoderPluginSource {
            err_callback: None,
            data,
            imp: self.imp.clone(),
        }))
    }
}

struct DecoderPluginSource {
    err_callback: Option<Callback<raplay::Error>>,
    data: OpaqueType,

    // MUST BE LAST in the drop order
    imp: Arc<DecoderPluginSourceImpl>,
}

impl DecoderPluginSource {
    fn get_errors(&self) -> anyhow::Result<()> {
        let mut fatal = None;
        for _ in 0..MAX_ERRS {
            let err = unsafe { (self.imp.err)(*self.data) };
            match CErrorType::from_id(err.typ) {
                Some(CErrorType::NoError) => break,
                Some(CErrorType::Fatal) if fatal.is_none() => {
                    fatal =
                        Some(anyhow!("plugin {}: {}", self.imp.name, err.msg));
                }
                _ => {
                    let err = anyhow!("plugin {}: {}", self.imp.name, err.msg);
                    if let Some(ref cb) = self.err_callback {
                        _ = cb.invoke(raplay::Error::Other(err));
                    } else {
                        warn!("{err}");
                    }
                }
            }
        }

        if let Some(err) = fatal {
            Err(err)
        } else {
            Ok(())
        }
    }
}

impl Source for DecoderPluginSource {
    fn set_err_callback(&mut self, err_callback: &Callback<raplay::Error>) {
        self.err_callback = Some(err_callback.clone());
    }

    fn init(&mut self, info: &DeviceConfig) -> anyhow::Result<()> {
        unsafe { (self.imp.set_config)(*self.data, &info.clone().into()) };
        self.get_errors()
    }

    fn read(
        &mut self,
        buffer: &mut SampleBufferMut,
    ) -> (usize, anyhow::Result<()>) {
        let size = match buffer {
            SampleBufferMut::I8(items) => unsafe {
                (self.imp.read)(
                    *self.data,
                    *items as *mut _ as *mut c_void,
                    items.len(),
                    CSampleFormat::I8,
                )
            },
            SampleBufferMut::I16(items) => unsafe {
                (self.imp.read)(
                    *self.data,
                    *items as *mut _ as *mut c_void,
                    items.len(),
                    CSampleFormat::I16,
                )
            },
            SampleBufferMut::I32(items) => unsafe {
                (self.imp.read)(
                    *self.data,
                    *items as *mut _ as *mut c_void,
                    items.len(),
                    CSampleFormat::I32,
                )
            },
            SampleBufferMut::I64(items) => unsafe {
                (self.imp.read)(
                    *self.data,
                    *items as *mut _ as *mut c_void,
                    items.len(),
                    CSampleFormat::I64,
                )
            },
            SampleBufferMut::U8(items) => unsafe {
                (self.imp.read)(
                    *self.data,
                    *items as *mut _ as *mut c_void,
                    items.len(),
                    CSampleFormat::U8,
                )
            },
            SampleBufferMut::U16(items) => unsafe {
                (self.imp.read)(
                    *self.data,
                    *items as *mut _ as *mut c_void,
                    items.len(),
                    CSampleFormat::U16,
                )
            },
            SampleBufferMut::U32(items) => unsafe {
                (self.imp.read)(
                    *self.data,
                    *items as *mut _ as *mut c_void,
                    items.len(),
                    CSampleFormat::U32,
                )
            },
            SampleBufferMut::U64(items) => unsafe {
                (self.imp.read)(
                    *self.data,
                    *items as *mut _ as *mut c_void,
                    items.len(),
                    CSampleFormat::U64,
                )
            },
            SampleBufferMut::F32(items) => unsafe {
                (self.imp.read)(
                    *self.data,
                    *items as *mut _ as *mut c_void,
                    items.len(),
                    CSampleFormat::F32,
                )
            },
            SampleBufferMut::F64(items) => unsafe {
                (self.imp.read)(
                    *self.data,
                    *items as *mut _ as *mut c_void,
                    items.len(),
                    CSampleFormat::F64,
                )
            },
            _ => return (0, Err(anyhow!("Unknown sample format."))),
        };

        (size, self.get_errors())
    }

    fn preferred_config(&mut self) -> Option<DeviceConfig> {
        if let Some(c) = self.imp.preferred_config {
            let cfg = unsafe { c(*self.data) };
            self.get_errors().ok()?;
            Some(cfg.into())
        } else {
            None
        }
    }

    fn volume(&mut self, volume: VolumeIterator) -> bool {
        if let Some(v) = self.imp.set_volume {
            unsafe { v(*self.data, volume.into()) };
            self.get_errors().is_ok()
        } else {
            false
        }
    }

    fn seek(&mut self, time: Duration) -> anyhow::Result<Timestamp> {
        if let Some(s) = self.imp.seek {
            let res = unsafe { s(*self.data, time.into()) };
            self.get_errors().map(|_| res.into())
        } else {
            Err(raplay::Error::Unsupported {
                component: "decoder plugin source",
                feature: "seeking",
            }
            .into())
        }
    }

    fn seek_by(
        &mut self,
        time: Duration,
        forward: bool,
    ) -> anyhow::Result<Timestamp> {
        if let Some(s) = self.imp.seek_by {
            let res = unsafe { s(*self.data, time.into(), forward) };
            self.get_errors().map(|_| res.into())
        } else {
            Source::seek_by(self, time, forward)
        }
    }

    fn get_time(&self) -> Option<Timestamp> {
        if let Some(t) = self.imp.get_time {
            let res = unsafe { t(*self.data) };
            self.get_errors().ok()?;
            Some(res.into())
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Debug)]
struct DecoderPluginConfig {
    version: u32,
    flags: DecoderPluginFlags,
}

bitflags! {
    #[repr(C)]
    #[derive(Debug)]
    struct DecoderPluginFlags: u32 {
        const VOLUME = 0x1;
        const CONFIG = 0x2;
        const SEEK = 0x4;
        const SEEK_BY = 0x8;
        const GET_TIME = 0x10;
    }
}

unsafe fn get_errors(
    d: *mut c_void,
    err: unsafe extern "C" fn(*mut c_void) -> CError,
    name: &str,
) -> anyhow::Result<()> {
    let mut fatal = None;
    for _ in 0..MAX_ERRS {
        let err = unsafe { err(d) };
        match CErrorType::from_id(err.typ) {
            Some(CErrorType::NoError) => break,
            Some(CErrorType::Fatal) if fatal.is_none() => {
                fatal = Some(anyhow!("plugin {name}: {}", err.msg));
            }
            _ => {
                warn!("plugin {name}: {}", err.msg);
            }
        }
    }

    if let Some(err) = fatal {
        Err(err)
    } else {
        Ok(())
    }
}
