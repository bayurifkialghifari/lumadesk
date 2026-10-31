use std::{error::Error, fs, io, path::Path};

use image::{
    Delay, Frame, Rgba, RgbaImage,
    codecs::gif::{GifEncoder, Repeat},
};
use lumadesk_core::{
    FrameBatchRequest, ImportOutcome, Library, MediaItem, SpritePlaybackMode, SpriteSequenceInput,
};
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

fn write_sprite_sheet(path: &Path) -> Result<(), image::ImageError> {
    let mut image = RgbaImage::new(4, 2);
    for (x, y, color) in [
        (0, 0, Rgba([255, 0, 0, 255])),
        (2, 0, Rgba([0, 255, 0, 255])),
        (0, 1, Rgba([0, 0, 255, 255])),
        (2, 1, Rgba([255, 255, 0, 255])),
    ] {
        image.put_pixel(x, y, color);
        image.put_pixel(x + 1, y, color);
    }
    image.save(path)
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

#[test]
fn extracts_sprite_sheet_cells_in_reading_order() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("sprite.png");
    write_sprite_sheet(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let item = imported(library.import_file(&source)?)?;
    library.set_sprite_sequence(&SpriteSequenceInput {
        media_id: item.id,
        columns: 2,
        rows: 2,
        frame_count: 4,
        frame_duration_ms: 75,
        playback_mode: SpritePlaybackMode::PingPong,
    })?;

    let batch = library.frame_batch(
        item.id,
        FrameBatchRequest {
            offset: 0,
            limit: 4,
        },
    )?;

    assert_eq!((batch.width, batch.height), (2, 1));
    assert_eq!(batch.total_frames, 4);
    assert_eq!(batch.total_duration_ms, 300);
    let colors = batch
        .frames
        .iter()
        .map(|frame| Ok(*image::open(&frame.path)?.to_rgba8().get_pixel(0, 0)))
        .collect::<Result<Vec<_>, image::ImageError>>()?;
    assert_eq!(
        colors,
        vec![
            Rgba([255, 0, 0, 255]),
            Rgba([0, 255, 0, 255]),
            Rgba([0, 0, 255, 255]),
            Rgba([255, 255, 0, 255]),
        ]
    );
    Ok(())
}

#[test]
fn rebuilds_an_in_memory_manifest_after_sprite_playback_changes() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("sprite.png");
    write_sprite_sheet(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let item = imported(library.import_file(&source)?)?;

    let source_batch = library.frame_batch(
        item.id,
        FrameBatchRequest {
            offset: 0,
            limit: 1,
        },
    )?;
    assert_eq!((source_batch.width, source_batch.height), (4, 2));

    library.set_sprite_sequence(&SpriteSequenceInput {
        media_id: item.id,
        columns: 2,
        rows: 2,
        frame_count: 4,
        frame_duration_ms: 75,
        playback_mode: SpritePlaybackMode::Loop,
    })?;
    let sprite_batch = library.frame_batch(
        item.id,
        FrameBatchRequest {
            offset: 0,
            limit: 4,
        },
    )?;

    assert_eq!((sprite_batch.width, sprite_batch.height), (2, 1));
    assert_eq!(sprite_batch.total_frames, 4);
    let colors = sprite_batch
        .frames
        .iter()
        .map(|frame| Ok(*image::open(&frame.path)?.to_rgba8().get_pixel(0, 0)))
        .collect::<Result<Vec<_>, image::ImageError>>()?;
    assert_eq!(
        colors,
        vec![
            Rgba([255, 0, 0, 255]),
            Rgba([0, 255, 0, 255]),
            Rgba([0, 0, 255, 255]),
            Rgba([255, 255, 0, 255]),
        ]
    );

    library.clear_sprite_sequence(item.id)?;
    let restored = library.frame_batch(
        item.id,
        FrameBatchRequest {
            offset: 0,
            limit: 1,
        },
    )?;
    assert_eq!((restored.width, restored.height), (4, 2));
    assert_eq!(restored.total_frames, 1);
    Ok(())
}

#[test]
fn clearing_generated_cache_preserves_imported_media() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("animated.gif");
    write_gif(&source)?;
    let data_root = temp.path().join("data");
    let mut library = Library::open(&data_root)?;
    let item = imported(library.import_file(&source)?)?;
    let managed_path = library.managed_media_path(item.id)?;
    let first_batch = library.frame_batch(
        item.id,
        FrameBatchRequest {
            offset: 0,
            limit: 2,
        },
    )?;
    assert!(
        first_batch
            .frames
            .iter()
            .all(|frame| Path::new(&frame.path).exists())
    );

    library.clear_generated_cache()?;

    assert!(managed_path.exists());
    assert!(data_root.join("frames").read_dir()?.next().is_none());
    assert!(data_root.join("thumbnails").read_dir()?.next().is_none());
    let rebuilt = library.frame_batch(
        item.id,
        FrameBatchRequest {
            offset: 0,
            limit: 2,
        },
    )?;
    assert_eq!(rebuilt.frames.len(), 2);
    Ok(())
}
