use crate::core::{
    Result,
    server::sub::{SetAll, SetPlaylist},
};

#[derive(Debug, Clone)]
pub enum SubMsg {
    SetAll(SetAll),
    SetPlaylist(SetPlaylist),
}

impl SubMsg {
    pub fn event(&self) -> Result<String> {
        match self {
            Self::SetAll(a) => Ok(format!(
                "event: set-all\ndata: {}\n\n",
                serde_json::ser::to_string(a)?
            )),
            Self::SetPlaylist(a) => Ok(format!(
                "event: set-playlist\ndata:{}\n\n",
                serde_json::ser::to_string(a)?
            )),
        }
    }
}
