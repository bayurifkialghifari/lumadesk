use std::{fs::File, io::BufReader, path::Path};

use image::{
    AnimationDecoder, ColorType, Frame, ImageFormat, ImageReader,
    codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder},
    save_buffer_with_format,
};

use crate::{CoreError, MediaFormat, SpriteSequence, media};

#[derive(Debug)]
pub(crate) struct DecodedFrames {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) durations_ms: Vec<u64>,
}

pub(crate) fn decode_to(
    source: &Path,
    format: MediaFormat,
    destination: &Path,
) -> Result<DecodedFrames, CoreError> {
    match format {
        MediaFormat::Gif => decode_gif(source, destination),
        MediaFormat::Apng => decode_apng(source, destination),
        MediaFormat::Webp => decode_webp(source, destination),
        MediaFormat::Png | MediaFormat::Jpeg => decode_static(source, destination),
        MediaFormat::Mp4
        | MediaFormat::Webm
        | MediaFormat::Mp3
        | MediaFormat::Wav
        | MediaFormat::Ogg => Err(CoreError::UnsupportedMedia),
    }
}

pub(crate) fn decode_sprite_sheet_to(
    source: &Path,
    sequence: &SpriteSequence,
    destination: &Path,
) -> Result<DecodedFrames, CoreError> {
    let image = ImageReader::open(source)?
        .with_guessed_format()?
        .decode()
        .map_err(media::image_error)?
        .into_rgba8();
    let frame_width = image.width() / sequence.columns;
    let frame_height = image.height() / sequence.rows;
    if frame_width == 0 || frame_height == 0 {
        return Err(CoreError::InvalidMedia(
            "The sprite grid is larger than the source image.".into(),
        ));
    }
    for index in 0..sequence.frame_count {
        let column = index % sequence.columns;
        let row = index / sequence.columns;
        let frame = image::imageops::crop_imm(
            &image,
            column * frame_width,
            row * frame_height,
            frame_width,
            frame_height,
        )
        .to_image();
        write_buffer(destination, index, &frame)?;
    }
    Ok(DecodedFrames {
        width: frame_width,
        height: frame_height,
        durations_ms: vec![u64::from(sequence.frame_duration_ms); sequence.frame_count as usize],
    })
}

fn decode_gif(source: &Path, destination: &Path) -> Result<DecodedFrames, CoreError> {
    let decoder =
        GifDecoder::new(BufReader::new(File::open(source)?)).map_err(media::image_error)?;
    write_frames(decoder.into_frames(), destination)
}

fn decode_apng(source: &Path, destination: &Path) -> Result<DecodedFrames, CoreError> {
    let decoder =
        PngDecoder::new(BufReader::new(File::open(source)?)).map_err(media::image_error)?;
    let decoder = decoder.apng().map_err(media::image_error)?;
    write_frames(decoder.into_frames(), destination)
}

fn decode_webp(source: &Path, destination: &Path) -> Result<DecodedFrames, CoreError> {
    let decoder =
        WebPDecoder::new(BufReader::new(File::open(source)?)).map_err(media::image_error)?;
    write_frames(decoder.into_frames(), destination)
}

fn decode_static(source: &Path, destination: &Path) -> Result<DecodedFrames, CoreError> {
    let image = ImageReader::open(source)?
        .with_guessed_format()?
        .decode()
        .map_err(media::image_error)?
        .into_rgba8();
    write_buffer(destination, 0, &image)?;
    Ok(DecodedFrames {
        width: image.width(),
        height: image.height(),
        durations_ms: vec![0],
    })
}

fn write_frames<'a>(
    frames: impl Iterator<Item = image::ImageResult<Frame>> + 'a,
    destination: &Path,
) -> Result<DecodedFrames, CoreError> {
    let mut durations_ms = Vec::new();
    let mut dimensions = None;
    for frame in frames {
        let frame = frame.map_err(media::image_error)?;
        let frame_delay_ms = media::delay_ms(frame.delay())?;
        let index = u32::try_from(durations_ms.len())
            .map_err(|_| CoreError::InvalidMedia("The frame count is too large.".into()))?;
        let buffer = frame.into_buffer();
        dimensions.get_or_insert((buffer.width(), buffer.height()));
        write_buffer(destination, index, &buffer)?;
        durations_ms.push(frame_delay_ms);
    }
    if durations_ms.is_empty() {
        return Err(CoreError::InvalidMedia(
            "The animation has no frames.".into(),
        ));
    }
    let (width, height) = dimensions
        .ok_or_else(|| CoreError::InvalidMedia("The animation has no frame dimensions.".into()))?;
    Ok(DecodedFrames {
        width,
        height,
        durations_ms,
    })
}

fn write_buffer(
    destination: &Path,
    index: u32,
    buffer: &image::RgbaImage,
) -> Result<(), CoreError> {
    save_buffer_with_format(
        destination.join(format!("{index:08}.png")),
        buffer.as_raw(),
        buffer.width(),
        buffer.height(),
        ColorType::Rgba8,
        ImageFormat::Png,
    )
    .map_err(media::image_error)
}
