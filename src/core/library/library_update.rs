//===========================================================================//
//                                   Public                                  //
//===========================================================================//

/// Describes how the library got updated.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Ord, Eq)]
pub enum LibraryUpdate {
    /// There is no change in the library.
    #[default]
    None = 0,
    /// Some metadata has changed.
    Metadata = 1,
    /// There is new data (new songs).
    NewData = 2,
    /// Some data were removed (songs).
    RemoveData = 3,
}
