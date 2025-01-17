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
    /// Some data was replaced. New data ids are same as old data ids. If you
    /// set library change to this state **DONT FORGET TO CALL**
    /// [`crate::core::UampApp::id_replace`].
    ReplaceData = 4,
}
