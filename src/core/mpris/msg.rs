use mpris_server::{Metadata, PlaybackStatus, Time, Volume};

#[derive(Debug, Clone)]
pub enum MprisMsg {
    PlaybackStatus(PlaybackStatus),
    Metadata(Metadata),
    Volume(Volume),
    Position(Time),
}
