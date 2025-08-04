use crate::core::{
    AppCtrl, Job, Msg, Result, UampApp,
    library::{LibraryLoadResult, SongId},
};

#[derive(Debug)]
pub enum JobMsg {
    LibraryLoad(Result<Option<LibraryLoadResult>>),
    Server(Result<()>),
    LibrarySave(Result<Vec<SongId>>),
    SystemPlayer,
}

impl UampApp {
    pub fn job_event(
        &mut self,
        ctrl: &mut AppCtrl,
        msg: JobMsg,
    ) -> Result<Vec<Msg>> {
        match msg {
            JobMsg::LibraryLoad(res) => {
                self.finish_library_load(ctrl, res?)?
            }
            JobMsg::LibrarySave(res) => self.finish_library_save_songs(res)?,
            JobMsg::Server(Err(e)) => {
                self.jobs.finish(Job::SERVER);
                return Err(e.prepend("Server ended unexpectedly."))?;
            }
            JobMsg::Server(Ok(_)) => {
                self.jobs.finish(Job::SERVER);
                if self.config.should_start_server() {
                    self.start_server(ctrl)?;
                }
            }
            JobMsg::SystemPlayer => {
                self.jobs.finish(Job::SYSTEM_PLAYER);
                if self.config.system_player() {
                    self.enable_system_player(ctrl);
                }
            }
        }
        Ok(vec![])
    }
}
