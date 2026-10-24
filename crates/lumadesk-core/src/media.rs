use std::{fs::File, io::BufReader, path::Path};

use image::{
    AnimationDecoder, Frame, ImageDecoder, ImageFormat, ImageReader,
    codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder},
};

use crate::{CoreError, MediaKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MediaMetadata {
    pub kind: MediaKind,
    pub width: u32,
    pub height: u32,
    pub frame_count: u32,
    pub duration_ms: u64,
}

pub(crate) fn inspect(path: &Path) -> Result<MediaMetadata, CoreError> {
    let reader = ImageReader::open(path)?.with_guessed_format()?;
    match reader.format().ok_or(CoreError::UnsupportedMedia)? {
        ImageFormat::Gif => inspect_gif(path),
        ImageFormat::Png => inspect_png(path),
        ImageFormat::WebP => inspect_webp(path),
        ImageFormat::Jpeg => inspect_static(path, MediaKind::Jpeg),
        _ => Err(CoreError::UnsupportedMedia),
    }
}

fn inspect_gif(path: &Path) -> Result<MediaMetadata, CoreError> {
    let decoder = GifDecoder::new(BufReader::new(File::open(path)?)).map_err(image_error)?;
    let (width, height) = decoder.dimensions();
    let (frame_count, duration_ms) = frame_metadata(decoder.into_frames())?;
    metadata(MediaKind::Gif, width, height, frame_count, duration_ms)
}

fn inspect_png(path: &Path) -> Result<MediaMetadata, CoreError> {
    let decoder = PngDecoder::new(BufReader::new(File::open(path)?)).map_err(image_error)?;
    if decoder.is_apng().map_err(image_error)? {
        let (width, height) = decoder.dimensions();
        let apng = decoder.apng().map_err(image_error)?;
        let (frame_count, duration_ms) = frame_metadata(apng.into_frames())?;
        metadata(MediaKind::Apng, width, height, frame_count, duration_ms)
    } else {
        inspect_static(path, MediaKind::Png)
    }
}

fn inspect_webp(path: &Path) -> Result<MediaMetadata, CoreError> {
    let decoder = WebPDecoder::new(BufReader::new(File::open(path)?)).map_err(image_error)?;
    if decoder.has_animation() {
        let (width, height) = decoder.dimensions();
        let (frame_count, duration_ms) = frame_metadata(decoder.into_frames())?;
        metadata(MediaKind::Webp, width, height, frame_count, duration_ms)
    } else {
        inspect_static(path, MediaKind::Webp)
    }
}

fn inspect_static(path: &Path, kind: MediaKind) -> Result<MediaMetadata, CoreError> {
    let image = ImageReader::open(path)?
        .with_guessed_format()?
        .decode()
        .map_err(image_error)?;
    metadata(kind, image.width(), image.height(), 1, 0)
}

fn frame_metadata<'a>(
    frames: impl Iterator<Item = image::ImageResult<Frame>> + 'a,
) -> Result<(u32, u64), CoreError> {
    let mut count = 0_u32;
    let mut duration = 0_u64;
    for frame in frames {
        let frame = frame.map_err(image_error)?;
        count = count
            .checked_add(1)
            .ok_or_else(|| CoreError::InvalidMedia("Jumlah frame terlalu besar.".into()))?;
        let (numerator, denominator) = frame.delay().numer_denom_ms();
        if denominator == 0 {
            return Err(CoreError::InvalidMedia("Durasi frame tidak valid.".into()));
        }
        let rounded = (u64::from(numerator) + u64::from(denominator) / 2) / u64::from(denominator);
        duration = duration
            .checked_add(rounded)
            .ok_or_else(|| CoreError::InvalidMedia("Durasi animasi terlalu panjang.".into()))?;
    }
    Ok((count, duration))
}

fn metadata(
    kind: MediaKind,
    width: u32,
    height: u32,
    frame_count: u32,
    duration_ms: u64,
) -> Result<MediaMetadata, CoreError> {
    if width == 0 || height == 0 || frame_count == 0 {
        return Err(CoreError::InvalidMedia(
            "Dimensi atau frame media kosong.".into(),
        ));
    }
    Ok(MediaMetadata {
        kind,
        width,
        height,
        frame_count,
        duration_ms,
    })
}

fn image_error(error: image::ImageError) -> CoreError {
    match error {
        image::ImageError::Unsupported(_) => CoreError::UnsupportedMedia,
        other => CoreError::InvalidMedia(other.to_string()),
    }
}
