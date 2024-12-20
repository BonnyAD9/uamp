use super::{Instance, Run};

//===========================================================================//
//                                   Public                                  //
//===========================================================================//

#[derive(Debug)]
/// Action that can be done with cli.
pub enum Action {
    /// Sends the given messages to a running instance.
    Instance(Instance),
    /// Runs uamp as detached process.
    RunDetached(Run),
    /// Configuration of uamp.
    Config(super::Config),
}
