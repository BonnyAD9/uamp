use audiotags::Tag;
use eyre::Result;

#[derive(Debug, Clone)]
pub struct Song {
    path: String,
    title: String,
    artist: String,
    album: String,
}

impl Song {
    pub fn from_path(path: String) -> Result<Self> {
        let tag = Tag::new().read_from_path(&path)?;
        Ok(Song {
            path,
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

    pub fn path(&self) -> &str {
        &self.path
    }
}
