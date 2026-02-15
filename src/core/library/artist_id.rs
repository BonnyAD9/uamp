use serde::Serialize;

use crate::ext::simpl;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ArtistId(String);

impl ArtistId {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self(simpl::new_str(name))
    }
}
