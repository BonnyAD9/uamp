use super::{Instance, Man, Run, Shell, internal::Internal};

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
    /// Shell features.
    Shell(Shell),
    /// Internal special features.
    Internal(Internal),
    /// See man pages.
    Man(Man),
}
