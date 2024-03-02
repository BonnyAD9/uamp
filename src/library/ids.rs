use serde::{Deserialize, Serialize};

macro_rules! make_ids {
    ($($id:ident),+ $(,)?) => {
        $(
            #[derive(
                Debug,
                Clone,
                Copy,
                Serialize,
                Deserialize,
                PartialEq
            )]
            pub struct $id(pub(super) usize);
        )*
    };
}

make_ids! { SongId, AlbumId, ArtistId }
