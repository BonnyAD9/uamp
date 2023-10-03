/// A library message
#[derive(Clone, Debug)]
pub enum Message {
    LoadEnded,
    SaveEnded,
    ImageLoadEnded,
    ImageShrinkEnded,
}
