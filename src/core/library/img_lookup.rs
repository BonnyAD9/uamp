use std::{
    borrow::Cow,
    fs::{self, create_dir_all},
    io::{Cursor, ErrorKind},
    path::{Path, PathBuf},
};

use audiotags::Tag;
use futures::executor::block_on;
use image::{DynamicImage, ImageReader};
use itertools::{Either, Itertools};
use log::warn;
use tokio::task::JoinHandle;
use unidecode::unidecode;

use crate::core::{
    Error, Result, RtAndle,
    config::CacheSize,
    library::Song,
    log_err,
    query::{Base, CmpType, ComposedFilter, Filter, FilterType, Query},
};

const MIN_IMG_SIZE: u64 = 125;

pub fn lookup_image(
    rt_path: Either<RtAndle, &Path>,
    cache: &Path,
    artist: &str,
    album: &str,
    size: CacheSize,
) -> Result<(PathBuf, Option<DynamicImage>)> {
    let name = filesan::escape_str(
        &simple_str(&format!("{artist} - {album}.jpg")),
        '_',
        filesan::Mode::ALL,
    );
    ImageLookup {
        rt_path,
        cache,
        artist,
        album,
        name: &name,
    }
    .lookup(size)
}

pub fn lookup_image_path_rt(
    rt: RtAndle,
    cache: &Path,
    artist: &str,
    album: &str,
    size: CacheSize,
) -> Result<PathBuf> {
    Ok(lookup_image(Either::Left(rt), cache, artist, album, size)?.0)
}

pub fn lookup_image_path_rt_thread(
    rt: RtAndle,
    cache: PathBuf,
    artist: String,
    album: String,
    size: CacheSize,
) -> JoinHandle<Result<PathBuf>> {
    tokio::task::spawn_blocking(move || {
        lookup_image_path_rt(rt, &cache, &artist, &album, size)
    })
}

pub fn lookup_image_data_song(
    song: &Song,
    cache: &Path,
    size: CacheSize,
) -> Result<DynamicImage> {
    let (path, img) = lookup_image(
        Either::Right(song.path()),
        cache,
        song.artist(),
        song.album(),
        size,
    )?;
    if let Some(img) = img {
        return Ok(img);
    }

    Ok(ImageReader::open(path)?.decode()?)
}

struct ImageLookup<'a> {
    rt_path: Either<RtAndle, &'a Path>,
    cache: &'a Path,
    artist: &'a str,
    album: &'a str,
    name: &'a str,
}

impl ImageLookup<'_> {
    fn lookup(
        &self,
        size: CacheSize,
    ) -> Result<(PathBuf, Option<DynamicImage>)> {
        let mut cached_path = self.cache.join(format!("cover{size}"));
        cached_path.push(self.name);

        if cached_path.try_exists()? {
            if cached_path.metadata()?.len() >= MIN_IMG_SIZE {
                return Ok((cached_path, None));
            }
            // Invalid image, try again.
            fs::remove_file(&cached_path)?;
        }

        let Some(size) = size.size() else {
            return self.lookup_full(&cached_path);
        };

        let img = match self.lookup(CacheSize::Full)? {
            (path, None) => self.cache_path_to(&path, &cached_path, size)?,
            (path, Some(img)) => {
                self.cache_img_to(img, &path, &cached_path, size)?
            }
        };

        Ok((cached_path, Some(img)))
    }

    fn lookup_full(
        &self,
        cache_to: &Path,
    ) -> Result<(PathBuf, Option<DynamicImage>)> {
        let rt = match self.rt_path {
            Either::Left(ref rt) => rt,
            Either::Right(path) => {
                return self.lookup_from_song_path(path, cache_to);
            }
        };

        let album = self.album.to_string();
        let artist = self.artist.to_string();
        let query = Query::new(
            vec![Base::Library],
            ComposedFilter::And(vec![
                ComposedFilter::Filter(Filter::new(
                    FilterType::Album(album),
                    CmpType::Lenient,
                )),
                ComposedFilter::Filter(Filter::new(
                    FilterType::Artist(artist),
                    CmpType::Lenient,
                )),
            ]),
            None,
            None,
        );
        let s = block_on(rt.request(move |app, _| {
            query
                .get_ids(&app.library, true, &app.player)
                .unwrap()
                .into_iter()
                .map(|i| app.library[i].path().to_owned())
                .collect_vec()
        }))?;

        for p in s {
            if let Some(res) = self.try_lookup_from_song_path(&p, cache_to)? {
                return Ok(res);
            }
        }

        Err(Error::NotFound("Couldn't find image.".into()))
    }

    fn lookup_from_song_path(
        &self,
        path: &Path,
        cache_to: &Path,
    ) -> Result<(PathBuf, Option<DynamicImage>)> {
        self.try_lookup_from_song_path(path, cache_to)?
            .ok_or_else(|| Error::NotFound("Couldn't find image.".into()))
    }

    fn try_lookup_from_song_path(
        &self,
        path: &Path,
        cache_to: &Path,
    ) -> Result<Option<(PathBuf, Option<DynamicImage>)>> {
        match self.try_lookup_tag(path, cache_to) {
            Ok(Some(r)) => {
                return Ok(Some((
                    cache_to.to_owned(),
                    Some(self.write_image_to(r, cache_to)?),
                )));
            }
            Ok(None) => {}
            Err(Error::AudioTag(e))
                if matches!(
                    e.inner(),
                    audiotags::Error::UnsupportedFormat(_)
                ) => {}
            Err(e) => return Err(e),
        }

        let Some(p) = path.parent() else {
            return Err(Error::io(std::io::Error::new(
                ErrorKind::NotFound,
                format!(
                    "Couldn't find parent directory of `{:?}`",
                    path.display()
                ),
            )));
        };

        let lookup_names: &[Cow<'static, str>] = &[
            // Uamp way
            filesan::escape_str(
                &format!(
                    "{} - {}",
                    simple_str(self.artist),
                    simple_str(self.album)
                ),
                '_',
                filesan::Mode::ALL,
            )
            .into(),
            // Winamp way
            filesan::replace_escape(self.album, '_', filesan::Mode::ALL)
                .into(),
            // Standard way if in separate folders.
            "cover".into(),
        ];
        let extensions = &["jpg", "jpeg", "png", "webp"];
        let path = lookup_names
            .iter()
            .flat_map(|n| extensions.iter().map(|e| n.to_string() + "." + e))
            .map(|n| p.join(n))
            .find(|p| p.exists());

        let Some(path) = path else {
            return Ok(None);
        };

        Ok(Some((
            cache_to.to_owned(),
            self.write_path_to(&path, None, cache_to)?,
        )))
    }

    fn try_lookup_tag(
        &self,
        path: &Path,
        cache_to: &Path,
    ) -> Result<Option<DynamicImage>> {
        let tag = Tag::new().read_from_path(path)?;
        let Some(img) = tag.album_cover() else {
            return Ok(None);
        };

        let img = ImageReader::new(Cursor::new(img.data))
            .with_guessed_format()?
            .decode()?;
        Ok(Some(self.write_image_to(img, cache_to)?))
    }

    fn cache_path_to(
        &self,
        src: &Path,
        dst: &Path,
        size: usize,
    ) -> Result<DynamicImage> {
        let img = ImageReader::open(src)?.decode()?;
        self.cache_img_to(img, src, dst, size)
    }

    fn cache_img_to(
        &self,
        img: DynamicImage,
        src_path: &Path,
        dst: &Path,
        size: usize,
    ) -> Result<DynamicImage> {
        // Don't upscale images
        if (img.width().max(img.height()) as usize) < size {
            return Ok(self.write_path_to(src_path, Some(img), dst)?.unwrap());
        }

        let (w, h) = if img.width() > img.height() {
            (size, img.height() as usize * size / img.width() as usize)
        } else {
            (img.width() as usize * size / img.height() as usize, size)
        };

        let img = image::imageops::resize(
            &img,
            w as u32,
            h as u32,
            image::imageops::FilterType::Triangle,
        );
        self.write_image_to(img.into(), dst)
    }

    fn write_image_to(
        &self,
        img: DynamicImage,
        dst: &Path,
    ) -> Result<DynamicImage> {
        make_parent(dst)?;
        let img = img.into_rgb8().into();
        looped_save(&img, dst)?;
        Ok(img)
    }

    fn write_path_to(
        &self,
        path: &Path,
        img: Option<DynamicImage>,
        dst: &Path,
    ) -> Result<Option<DynamicImage>> {
        if path.extension() != dst.extension() {
            return if let Some(img) = img {
                Ok(Some(self.write_image_to(img, dst)?))
            } else {
                let img = ImageReader::open(path)?.decode()?;
                Ok(Some(self.write_image_to(img, dst)?))
            };
        }
        let dst = dst.with_extension(path.extension().unwrap_or_default());
        make_parent(&dst)?;
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(path, &dst)?;
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(path, &dst)?;
        }
        Ok(None)
    }
}

fn make_parent(p: &Path) -> Result<()> {
    if let Some(p) = p.parent() {
        create_dir_all(p)?;
    }
    Ok(())
}

fn simple_str(s: &str) -> String {
    unidecode(s).to_ascii_lowercase()
}

/// Save the image and verify it.
///
/// This is done because for some reason, the save sometimes failes.
fn looped_save(img: &DynamicImage, dst: &Path) -> Result<()> {
    const MAX_TRIES: usize = 4;

    for i in 1..=MAX_TRIES {
        img.save(dst)?;
        if dst.exists() && dst.metadata()?.len() >= MIN_IMG_SIZE {
            return Ok(());
        }
        warn!("Failed to save image to {} {i} times.", dst.display());
        log_err("Failed to remove file.", fs::remove_file(dst));
    }

    Err(Error::Unexpected("Failed to save image.".into()))
}
