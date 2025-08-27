use core::fmt::Debug;

use tokio::sync::oneshot;

use crate::core::{
    AppCtrl, Error, IdControlMsg, JobMsg, RtAndle, RtHandle, UampApp,
};

use super::{
    AnyControlMsg, ControlMsg, DataControlMsg, MessageDelegate, Result,
    config::ConfigMsg, player::PlayerMsg,
};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Event messages in uamp
#[allow(missing_debug_implementations)]
#[derive(Debug, Default)]
pub enum Msg {
    /// Some simple messages.
    Control(ControlMsg),
    /// More complicated messages.
    DataControl(Box<DataControlMsg>),
    /// Messages from cooperating client referencing song ids.
    IdControl(IdControlMsg),
    /// Player messges handled by the player.
    Player(PlayerMsg),
    /// Dellegate the message.
    Delegate(Box<dyn MessageDelegate>),
    /// Message for configuration.
    Config(ConfigMsg),
    Job(JobMsg),
    /// Nothing, just do the usual updates.
    #[default]
    None,
}

impl UampApp {
    /// Handle the message event.
    pub(in crate::core) fn msg_event(
        &mut self,
        ctrl: &mut AppCtrl,
        msg: Msg,
    ) -> Result<Vec<Msg>> {
        let mut res = match msg {
            Msg::Control(msg) => self.control_event(ctrl, msg)?,
            Msg::DataControl(msg) => self.data_control_event(ctrl, *msg)?,
            Msg::IdControl(msg) => self.id_control_event(msg)?,
            Msg::Player(msg) => self.player_event(msg),
            Msg::Delegate(d) => d.update(self, ctrl)?,
            Msg::Config(msg) => self.config_event(ctrl, msg)?,
            Msg::Job(msg) => self.job_event(ctrl, msg)?,
            Msg::None => vec![],
        };

        res.splice(
            0..0,
            self.player.get_playlist_action(
                self.config.default_playlist_end_action().as_ref(),
            ),
        );
        Ok(res)
    }
}

impl From<ControlMsg> for Msg {
    fn from(value: ControlMsg) -> Self {
        Self::Control(value)
    }
}

impl From<DataControlMsg> for Msg {
    fn from(value: DataControlMsg) -> Self {
        Self::DataControl(Box::new(value))
    }
}

impl From<Box<DataControlMsg>> for Msg {
    fn from(value: Box<DataControlMsg>) -> Self {
        Self::DataControl(value)
    }
}

impl From<AnyControlMsg> for Msg {
    fn from(value: AnyControlMsg) -> Self {
        match value {
            AnyControlMsg::Control(ctrl) => Self::Control(ctrl),
            AnyControlMsg::Data(data) => Self::DataControl(data),
        }
    }
}

impl From<PlayerMsg> for Msg {
    fn from(value: PlayerMsg) -> Self {
        Self::Player(value)
    }
}

impl From<ConfigMsg> for Msg {
    fn from(value: ConfigMsg) -> Self {
        Msg::Config(value)
    }
}

impl From<JobMsg> for Msg {
    fn from(value: JobMsg) -> Self {
        Msg::Job(value)
    }
}

impl RtHandle {
    pub async fn request<T: Send + 'static, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut UampApp, &mut AppCtrl) -> T + 'static + Sync + Send,
    {
        let (rsend, rrecv) = oneshot::channel();
        self.msg(Msg::fn_delegate(move |app, ctrl| {
            _ = rsend.send(f(app, ctrl));
            Ok(vec![])
        }));
        rrecv.await.map_err(|_| {
            Error::Unexpected("Failed to receive data with request.".into())
        })
    }
}

impl RtAndle {
    pub async fn request<T: Send + 'static, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut UampApp, &mut AppCtrl) -> T + 'static + Sync + Send,
    {
        let (rsend, rrecv) = oneshot::channel();
        self.msg(Msg::fn_delegate(move |app, ctrl| {
            _ = rsend.send(f(app, ctrl));
            Ok(vec![])
        }));
        rrecv.await.map_err(|_| {
            Error::Unexpected("Failed to receive data with request.".into())
        })
    }
}
