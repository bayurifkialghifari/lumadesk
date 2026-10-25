use std::{error::Error, fs, io, path::Path};

use image::{
    Delay, Frame, Rgba, RgbaImage,
    codecs::gif::{GifEncoder, Repeat},
};
use lumadesk_core::{FrameBatchRequest, ImportOutcome, Library, MediaItem};
use tempfile::TempDir;

fn write_gif(path: &Path) -> Result<(), Box<dyn Error>> {
    let file = fs::File::create(path)?;
    let mut encoder = GifEncoder::new(file);
    encoder.set_repeat(Repeat::Infinite)?;
    for (color, delay_ms) in [([255, 0, 0, 255], 40), ([0, 0, 255, 255], 90)] {
        let pixels = RgbaImage::from_pixel(3, 2, Rgba(color));
        encoder.encode_frame(Frame::from_parts(
            pixels,
            0,
            0,
            Delay::from_numer_denom_ms(delay_ms, 1),
        ))?;
    }
    Ok(())
}

fn imported(outcome: ImportOutcome) -> Result<MediaItem, io::Error> {
    match outcome {
        ImportOutcome::Imported { item } => Ok(item),
        ImportOutcome::Duplicate { .. } => Err(io::Error::other(
            "the first fixture must not be detected as a duplicate",
        )),
    }
}

#[test]
fn decodes_animation_into_bounded_batches_with_original_timing() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("blink.gif");
    write_gif(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let item = imported(library.import_file(&source)?)?;

    let first = library.frame_batch(
        item.id,
        FrameBatchRequest {
            offset: 0,
            limit: 1,
        },
    )?;
    let second = library.frame_batch(
        item.id,
        FrameBatchRequest {
            offset: 1,
            limit: 1,
        },
    )?;

    assert_eq!(first.total_frames, 2);
    assert_eq!(first.total_duration_ms, 130);
    assert_eq!(first.frames[0].duration_ms, 40);
    assert_eq!(second.frames[0].duration_ms, 90);
    assert_eq!(
        image::open(&second.frames[0].path)?
            .to_rgba8()
            .get_pixel(0, 0),
        &Rgba([0, 0, 255, 255]),
    );
    Ok(())
}

#[test]
fn reuses_complete_disk_cache_after_reopening_library() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("cache.gif");
    let data = temp.path().join("data");
    write_gif(&source)?;
    let mut library = Library::open(&data)?;
    let item = imported(library.import_file(&source)?)?;
    let managed = library.managed_media_path(item.id)?;
    library.frame_batch(
        item.id,
        FrameBatchRequest {
            offset: 0,
            limit: 2,
        },
    )?;
    drop(library);
    fs::remove_file(managed)?;

    let mut reopened = Library::open(&data)?;
    let batch = reopened.frame_batch(
        item.id,
        FrameBatchRequest {
            offset: 0,
            limit: 2,
        },
    )?;

    assert_eq!(batch.frames.len(), 2);
    assert!(
        batch
            .frames
            .iter()
            .all(|frame| Path::new(&frame.path).is_file())
    );
    Ok(())
}

#[test]
fn rejects_unbounded_frame_batch_requests() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("limit.gif");
    write_gif(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let item = imported(library.import_file(&source)?)?;

    let result = library.frame_batch(
        item.id,
        FrameBatchRequest {
            offset: 0,
            limit: 65,
        },
    );

    assert!(result.is_err());
    Ok(())
}
