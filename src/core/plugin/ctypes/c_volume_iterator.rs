use raplay::source::VolumeIterator;

#[repr(C)]
#[derive(Debug, Default)]
pub struct CVolumeIterator {
    /// If false, the volume is set in base
    linear: bool,
    base: f32,
    step: f32,
    cur_count: i32,
    target_count: i32,
    multiplier: f32,
    channel_count: usize,
    cur_channel: usize,
}

impl From<VolumeIterator> for CVolumeIterator {
    fn from(value: VolumeIterator) -> Self {
        match value {
            VolumeIterator::Constant(base) => Self {
                base,
                ..Default::default()
            },
            VolumeIterator::Linear {
                base,
                step,
                cur_count,
                target_count,
                multiplier,
                channel_count,
                cur_channel,
            } => Self {
                linear: true,
                base,
                step,
                cur_count,
                target_count,
                multiplier,
                channel_count,
                cur_channel,
            },
        }
    }
}
