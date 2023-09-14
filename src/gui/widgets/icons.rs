///! this module contains constants with svg icons
///! the icons are of type [`SvgData`], that implements [`Into<iced::svg::Handle>`]
///! so it can be used as simply as:
///! ```
///! let element = iced::widget::svg(icons::ICON);
///! ```
use iced_core::svg::Handle;

/// icon similar to '^', it is used for the top button in scrollbar
pub const POINT_UP: SvgData = SvgData::new(
    include_bytes!("../../../assets/svg/point_up.svg").as_slice(),
);

/// icon similar to 'v', it is used for the bottom button in scrollbar
pub const POINT_DOWN: SvgData = SvgData::new(
    include_bytes!("../../../assets/svg/point_down.svg").as_slice(),
);

/// filled triangle pointing to the right '|>', it is used as the play button
pub const PLAY: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/play.svg").as_slice());

/// two vertical lines '||', it is used as the pause button
pub const PAUSE: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/pause.svg").as_slice());

/// filled square '[]', it is used as the stop button
pub const _STOP: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/stop.svg").as_slice());

/// triangle pointing to the right and a line '|>|', it is used as the next button
pub const NEXT: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/next.svg").as_slice());

/// line and a triangle pointing to the left '|<|', it is used as the previous button
pub const PREVIOUS: SvgData = SvgData::new(
    include_bytes!("../../../assets/svg/previous.svg").as_slice(),
);

/// icon signifying volume
pub const VOLUME_0: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/volume_0.svg").as_slice());

/// icon signifying volume
pub const VOLUME_1: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/volume_1.svg").as_slice());

/// icon signifying volume
pub const VOLUME_2: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/volume_2.svg").as_slice());

/// icon signifying volume
pub const VOLUME_3: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/volume_3.svg").as_slice());

/// icon signifying volume
pub const VOLUME_4: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/volume_4.svg").as_slice());

/// icon signifying mute (crossed volume)
pub const NO_VOLUME_0: SvgData = SvgData::new(
    include_bytes!("../../../assets/svg/no_volume_0.svg").as_slice(),
);

/// icon signifying mute (crossed volume)
pub const NO_VOLUME_1: SvgData = SvgData::new(
    include_bytes!("../../../assets/svg/no_volume_1.svg").as_slice(),
);

/// icon signifying mute (crossed volume)
pub const NO_VOLUME_2: SvgData = SvgData::new(
    include_bytes!("../../../assets/svg/no_volume_2.svg").as_slice(),
);

/// icon signifying mute (crossed volume)
pub const NO_VOLUME_3: SvgData = SvgData::new(
    include_bytes!("../../../assets/svg/no_volume_3.svg").as_slice(),
);

/// icon signifying mute (crossed volume)
pub const NO_VOLUME_4: SvgData = SvgData::new(
    include_bytes!("../../../assets/svg/no_volume_4.svg").as_slice(),
);

/// Icon signifying fast forward, '|>|>'
pub const FAST_FORWARD: SvgData = SvgData::new(
    include_bytes!("../../../assets/svg/fast_forward.svg").as_slice(),
);

/// Icon signifying fast rewind, '<|<|'
pub const REWIND: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/rewind.svg").as_slice());

/// Icon signifying fast rewind, '<|<|'
pub const _CIRCLE: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/circle.svg").as_slice());

pub const _UAMP_LIGHT: SvgData = SvgData::new(
    include_bytes!("../../../assets/svg/icon_light.svg").as_slice(),
);

pub const UAMP: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/icon.svg").as_slice());

pub const TRASH: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/trash.svg").as_slice());

pub const ADD: SvgData =
    SvgData::new(include_bytes!("../../../assets/svg/add.svg").as_slice());

pub fn volume(volume: f32) -> SvgData {
    match volume {
        v if v <= 0. => VOLUME_0,
        v if v < 1. / 3. => VOLUME_1,
        v if v < 2. / 3. => VOLUME_2,
        v if v >= 1. => VOLUME_4,
        _ => VOLUME_3,
    }
}

pub fn no_volume(volume: f32) -> SvgData {
    match volume {
        v if v <= 0. => NO_VOLUME_0,
        v if v < 1. / 3. => NO_VOLUME_1,
        v if v < 2. / 3. => NO_VOLUME_2,
        v if v >= 1. => NO_VOLUME_4,
        _ => NO_VOLUME_3,
    }
}

/// contains svg data, can be created with const function, implements [`Into<iced::svg::Handle>`]
#[derive(Clone, Copy)]
pub struct SvgData(&'static [u8]);

impl SvgData {
    /// creates SVG from svg string as bytes
    pub const fn new(data: &'static [u8]) -> Self {
        Self { 0: data }
    }
}

impl Into<Handle> for SvgData {
    fn into(self) -> Handle {
        Handle::from_memory(self.0)
    }
}
