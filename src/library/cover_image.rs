use std::io::Cursor;

use iced_core::image::Handle;
use image::{
    codecs::png::PngEncoder,
    imageops::{resize, FilterType},
    io::Reader as ImageReader,
    ImageBuffer, Rgba,
};

pub const SMALL_SIZE: u32 = 100;

#[derive(Debug, Clone)]
pub struct CoverImage {
    pub inner: Handle,
    pub small: Option<Handle>,
}

impl CoverImage {
    pub fn new(full: Handle, small: Option<Handle>) -> Self {
        Self { inner: full, small }
    }

    pub fn _from_data(data: Vec<u8>) -> Self {
        Self {
            inner: Handle::from_memory(data),
            small: None,
        }
    }

    pub fn as_andle(&self) -> Handle {
        self.inner.clone()
    }

    pub fn as_small(&self) -> Option<Handle> {
        self.small.clone()
    }

    /// Creates the small image version, returns Some if there is new image
    pub fn make_thumbnail(&mut self) -> Option<Handle> {
        if self.small.is_some() {
            return None;
        }

        let mut data = Vec::new();
        let enc = PngEncoder::new(&mut data);

        match self.inner.data() {
            iced_core::image::Data::Path(p) => resize(
                &ImageReader::open(p)
                    .ok()?
                    .with_guessed_format()
                    .ok()?
                    .decode()
                    .ok()?,
                SMALL_SIZE,
                SMALL_SIZE,
                FilterType::Triangle,
            )
            .write_with_encoder(enc)
            .ok()?,
            iced_core::image::Data::Bytes(b) => resize(
                &ImageReader::new(Cursor::new(b))
                    .with_guessed_format()
                    .ok()?
                    .decode()
                    .ok()?,
                SMALL_SIZE,
                SMALL_SIZE,
                FilterType::Triangle,
            )
            .write_with_encoder(enc)
            .ok()?,
            iced_core::image::Data::Rgba {
                width,
                height,
                pixels,
            } => {
                let img: ImageBuffer<Rgba<_>, _> =
                    ImageBuffer::from_raw(*width, *height, pixels.as_ref())?;
                resize(&img, SMALL_SIZE, SMALL_SIZE, FilterType::Triangle)
                    .write_with_encoder(enc)
                    .ok()?
            }
        }

        self.small = Some(Handle::from_memory(data));

        self.as_small()
    }
}
