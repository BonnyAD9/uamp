use iced_native::svg::Handle;

pub const POINT_UP: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/point_up.svg").as_slice());

pub const POINT_DOWN: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/point_down.svg").as_slice());

pub const PLAY: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/play.svg").as_slice());

pub const PAUSE: SvgData =
    SvgData::new(include_bytes!("../../assets/svg/pause.svg").as_slice());

#[derive(Clone, Copy)]
pub struct SvgData(&'static [u8]);

impl SvgData {
    pub const fn new(data: &'static [u8]) -> Self {
        Self { 0: data }
    }
}

impl Into<Handle> for SvgData {
    fn into(self) -> Handle {
        Handle::from_memory(self.0)
    }
}
