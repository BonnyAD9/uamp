use crate::core::{AnyControlMsg, server::ReqMsg};

#[derive(Debug)]
pub enum SndMsg {
    Ctrl(AnyControlMsg),
    Req(ReqMsg),
}

impl From<AnyControlMsg> for SndMsg {
    fn from(value: AnyControlMsg) -> Self {
        Self::Ctrl(value)
    }
}

impl From<ReqMsg> for SndMsg {
    fn from(value: ReqMsg) -> Self {
        Self::Req(value)
    }
}
