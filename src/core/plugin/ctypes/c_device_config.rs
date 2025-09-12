use raplay::source::DeviceConfig;

use crate::core::plugin::ctypes::CSampleFormat;

#[repr(C)]
#[derive(Debug)]
pub struct CDeviceConfig {
    pub channel_count: u32,
    pub sample_rate: u32,
    pub sample_format: CSampleFormat,
}

impl From<DeviceConfig> for CDeviceConfig {
    fn from(value: raplay::source::DeviceConfig) -> Self {
        Self {
            channel_count: value.channel_count,
            sample_rate: value.sample_rate,
            sample_format: value.sample_format.into(),
        }
    }
}

impl From<CDeviceConfig> for DeviceConfig {
    fn from(value: CDeviceConfig) -> Self {
        DeviceConfig {
            channel_count: value.channel_count,
            sample_rate: value.sample_rate,
            sample_format: value.sample_format.into(),
        }
    }
}
