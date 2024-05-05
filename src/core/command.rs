#[derive(Clone, Debug, Copy)]
pub enum Command {
    None,
    Exit,
}

#[derive(Clone, Debug)]
pub enum ComMsg<Msg> {
    Command(Command),
    Msg(Msg),
}

impl<Msg> ComMsg<Msg> {
    pub fn none() -> Self {
        Self::Command(Command::None)
    }
}
