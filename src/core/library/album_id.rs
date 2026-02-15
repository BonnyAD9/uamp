use serde::Serialize;

use crate::ext::simpl;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct AlbumId(String);

impl AlbumId {
    pub fn new(artist: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        let mut id = simpl::new_str(artist);
        id.push('\t');
        simpl::to_str(name, &mut id);
        Self(id)
    }
}
