///! this module contains constants with svg icons
///! the icons are of type [`SvgData`], that implements [`Into<iced::svg::Handle>`]
///! so it can be used as simply as:
///! ```
///! let element = iced::widget::svg(icons::ICON);
///! ```
use iced_core::svg::Handle;

/// icon similar to '^', it is used for the top button in scrollbar
pub const POINT_UP: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/point_up.svg").as_slice());

/// icon similar to 'v', it is used for the bottom button in scrollbar
pub const POINT_DOWN: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/point_down.svg").as_slice());

/// filled triangle pointing to the right '|>', it is used as the play button
pub const PLAY: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/play.svg").as_slice());

/// two vertical lines '||', it is used as the pause button
pub const PAUSE: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/pause.svg").as_slice());

/// filled square '[]', it is used as the stop button
pub const STOP: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/stop.svg").as_slice());

/// line and a triangle pointing to the right '||>', it is used as the next button
pub const NEXT: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/next.svg").as_slice());

/// triangle pointing to the left and line '<||', it is used as the previous button
pub const PREVIOUS: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/previous.svg").as_slice());

/// icon signifying volume
pub const VOLUME: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/volume.svg").as_slice());

/// icon signifying mute (crossed volume)
pub const NO_VOLUME: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/no_volume.svg").as_slice());

pub const FAST_FORWARD: SvgData = SvgData::new(
    include_bytes!("../../assets/svg/fast_forward.svg").as_slice(),
);

pub const REWIND: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/rewind.svg").as_slice());

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
