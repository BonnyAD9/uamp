use serde::{Deserialize, Serialize};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SongPosSave {
    #[default]
    Never,
    OnClose,
    Always,
}

impl SongPosSave {
    #[inline]
    pub fn save(&self, closing: bool) -> bool {
        match self {
            Self::Always => true,
            Self::OnClose => closing,
            Self::Never => false,
        }
    }
}
