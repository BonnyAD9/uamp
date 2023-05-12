use audiotags::Tag;
use eyre::Result;

pub struct Song {
    title: String,
    artist: String,
    album: String,
}

impl Song {
    pub fn from_path(path: &str) -> Result<Self> {
        let tag = Tag::new().read_from_path(path)?;
        Ok(Song {
            title: tag.title().unwrap_or("<unknown title>").to_owned(),
            artist: tag.artist().unwrap_or("<unknown artist>").to_owned(),
            album: tag.album_title().unwrap_or("<unknown album>").to_owned(),
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn artist(&self) -> &str {
        &self.artist
    }

    pub fn album(&self) -> &str {
        &self.album
    }
}
