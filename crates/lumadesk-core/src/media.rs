use std::{fs::File, io::BufReader, path::Path};

use image::{
    AnimationDecoder, Delay, Frame, ImageDecoder, ImageFormat, ImageReader,
    codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder},
};

use crate::{AssetKind, CoreError, MediaFormat};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MediaMetadata {
    pub asset_kind: AssetKind,
    pub format: MediaFormat,
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
        ImageFormat::Jpeg => inspect_static(path, MediaFormat::Jpeg),
        _ => Err(CoreError::UnsupportedMedia),
    }
}

fn inspect_gif(path: &Path) -> Result<MediaMetadata, CoreError> {
    let decoder = GifDecoder::new(BufReader::new(File::open(path)?)).map_err(image_error)?;
    let (width, height) = decoder.dimensions();
    let (frame_count, duration_ms) = frame_metadata(decoder.into_frames())?;
    metadata(MediaFormat::Gif, width, height, frame_count, duration_ms)
}

fn inspect_png(path: &Path) -> Result<MediaMetadata, CoreError> {
    let decoder = PngDecoder::new(BufReader::new(File::open(path)?)).map_err(image_error)?;
    if decoder.is_apng().map_err(image_error)? {
        let (width, height) = decoder.dimensions();
        let apng = decoder.apng().map_err(image_error)?;
        let (frame_count, duration_ms) = frame_metadata(apng.into_frames())?;
        metadata(MediaFormat::Apng, width, height, frame_count, duration_ms)
    } else {
        inspect_static(path, MediaFormat::Png)
    }
}

fn inspect_webp(path: &Path) -> Result<MediaMetadata, CoreError> {
    let decoder = WebPDecoder::new(BufReader::new(File::open(path)?)).map_err(image_error)?;
    if decoder.has_animation() {
        let (width, height) = decoder.dimensions();
        let (frame_count, duration_ms) = frame_metadata(decoder.into_frames())?;
        metadata(MediaFormat::Webp, width, height, frame_count, duration_ms)
    } else {
        inspect_static(path, MediaFormat::Webp)
    }
}

fn inspect_static(path: &Path, format: MediaFormat) -> Result<MediaMetadata, CoreError> {
    let image = ImageReader::open(path)?
        .with_guessed_format()?
        .decode()
        .map_err(image_error)?;
    metadata(format, image.width(), image.height(), 1, 0)
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
            .ok_or_else(|| CoreError::InvalidMedia("The frame count is too large.".into()))?;
        let rounded = delay_ms(frame.delay())?;
        duration = duration
            .checked_add(rounded)
            .ok_or_else(|| CoreError::InvalidMedia("The animation duration is too long.".into()))?;
    }
    Ok((count, duration))
}

fn metadata(
    format: MediaFormat,
    width: u32,
    height: u32,
    frame_count: u32,
    duration_ms: u64,
) -> Result<MediaMetadata, CoreError> {
    if width == 0 || height == 0 || frame_count == 0 {
        return Err(CoreError::InvalidMedia(
            "The media dimensions or frame list are empty.".into(),
        ));
    }
    Ok(MediaMetadata {
        asset_kind: if frame_count > 1 {
            AssetKind::Animation
        } else {
            AssetKind::Still
        },
        format,
        width,
        height,
        frame_count,
        duration_ms,
    })
}

pub(crate) fn delay_ms(delay: Delay) -> Result<u64, CoreError> {
    let (numerator, denominator) = delay.numer_denom_ms();
    if denominator == 0 {
        return Err(CoreError::InvalidMedia(
            "The frame duration is invalid.".into(),
        ));
    }
    Ok((u64::from(numerator) + u64::from(denominator) / 2) / u64::from(denominator))
}

pub(crate) fn image_error(error: image::ImageError) -> CoreError {
    match error {
        image::ImageError::Unsupported(_) => CoreError::UnsupportedMedia,
        other => CoreError::InvalidMedia(other.to_string()),
    }
}
