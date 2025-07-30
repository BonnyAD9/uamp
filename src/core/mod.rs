mod control;
mod err;
mod jobs;
mod message_delegate;
mod msg;
mod play_msg;
mod server;
mod state;
mod uamp_app;

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

pub mod config;
pub mod library;
pub mod messenger;
pub mod player;
pub mod query;

#[cfg(unix)]
mod mpris;

pub use self::{
    control::*, err::*, jobs::*, message_delegate::*, msg::*, play_msg::*,
    state::*, uamp_app::*,
};

pub type AppCtrl<'a> = crate::env::AppCtrl<'a, Msg, Error>;
pub type RtHandle = crate::env::rt::Handle<Msg, Error>;
pub type RtAndle = crate::env::rt::Andle<Msg, Error>;
