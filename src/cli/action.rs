use super::{Instance, Run};

/// Action that can be done with cli
pub enum Action {
    /// Sends the given message
    Instance(Instance),
    RunDetached(Run),
}
