use std::io::{BufRead, Read, Seek};

use iced_core::image::Handle;
use image::io::Reader as ImageReader;

use crate::core::err::Result;

#[derive(Debug, Clone)]
pub struct CoverImage {
    inner: Handle,
}

impl CoverImage {
    pub fn _from_reader<R>(reader: R) -> Result<Self>
    where
        R: Read + Seek + BufRead,
    {
        let img = ImageReader::new(reader)
        .with_guessed_format()?
        .decode()?
        .into_rgba8();
        let width = img.width();
        let height = img.height();
        Ok(Self {
            inner: Handle::from_pixels(width, height, img.into_raw()),
        })
    }

    pub fn from_data(data: Vec<u8>) -> Self {
        Self {
            inner: Handle::from_memory(data)
        }
    }

    pub fn as_andle(&self) -> Handle {
        self.inner.clone()
    }
}
