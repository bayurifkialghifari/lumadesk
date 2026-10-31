use std::{error::Error, fs, io, path::Path};

use image::{DynamicImage, Rgba, RgbaImage};
use lumadesk_core::{
    ImportOutcome, Library, MediaItem, SpritePlaybackMode, SpriteSequenceInput, VariantKind,
};
use tempfile::TempDir;

fn write_png(path: &Path) -> Result<(), image::ImageError> {
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, Rgba([24, 190, 255, 255]))).save(path)
}

fn imported(outcome: ImportOutcome) -> Result<MediaItem, io::Error> {
    match outcome {
        ImportOutcome::Imported { item } => Ok(item),
        ImportOutcome::Duplicate { .. } => Err(io::Error::other(
            "the first fixture must not be detected as a duplicate",
        )),
    }
}

fn write_mask(data_root: &Path, relative_path: &str) -> Result<(), io::Error> {
    let path = data_root.join(relative_path);
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| io::Error::other("mask has no parent"))?,
    )?;
    fs::write(path, b"mask")
}

#[test]
fn generated_variants_are_upserted_and_removed_by_literal_prefix() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("subject.png");
    write_png(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let item = imported(library.import_file(&source)?)?;

    let original = library.upsert_media_variant(
        item.id,
        VariantKind::SegmentationMask,
        "masks/fast_subject/first.png",
        4,
        4,
    )?;
    let updated = library.upsert_media_variant(
        item.id,
        VariantKind::SegmentationMask,
        "masks/fast_subject/first.png",
        8,
        6,
    )?;
    library.upsert_media_variant(
        item.id,
        VariantKind::SegmentationMask,
        "masks/fastXsubject/second.png",
        4,
        4,
    )?;

    assert_eq!(updated.id, original.id);
    assert_eq!((updated.width, updated.height), (8, 6));
    assert_eq!(
        library.remove_variants_by_prefix(VariantKind::SegmentationMask, "masks/fast_subject/",)?,
        ["masks/fast_subject/first.png"]
    );
    assert_eq!(library.media_variants(item.id)?.len(), 1);
    assert!(
        library
            .upsert_media_variant(
                item.id,
                VariantKind::SegmentationMask,
                "../outside.png",
                4,
                4,
            )
            .is_err()
    );
    Ok(())
}

#[test]
fn sprite_changes_remove_cached_masks_and_variant_records() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let data_root = temp.path().join("data");
    let source = temp.path().join("sheet.png");
    write_png(&source)?;
    let mut library = Library::open(&data_root)?;
    let item = imported(library.import_file(&source)?)?;
    library.set_sprite_sequence(&SpriteSequenceInput {
        media_id: item.id,
        columns: 2,
        rows: 2,
        frame_count: 4,
        frame_duration_ms: 80,
        playback_mode: SpritePlaybackMode::Loop,
    })?;
    let relative_path = format!("masks/fast/{}/source/00000000.png", item.id);
    write_mask(&data_root, &relative_path)?;
    library.upsert_media_variant(item.id, VariantKind::SegmentationMask, &relative_path, 2, 2)?;

    library.set_sprite_sequence(&SpriteSequenceInput {
        media_id: item.id,
        columns: 4,
        rows: 1,
        frame_count: 4,
        frame_duration_ms: 120,
        playback_mode: SpritePlaybackMode::PingPong,
    })?;

    assert!(!data_root.join(relative_path).exists());
    assert_eq!(library.media_variants(item.id)?, Vec::new());
    Ok(())
}

#[test]
fn clearing_generated_cache_removes_mask_files_and_records() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let data_root = temp.path().join("data");
    let source = temp.path().join("subject.png");
    write_png(&source)?;
    let mut library = Library::open(&data_root)?;
    let item = imported(library.import_file(&source)?)?;
    let relative_path = format!("masks/quality/{}/source/00000000.png", item.id);
    write_mask(&data_root, &relative_path)?;
    library.upsert_media_variant(item.id, VariantKind::SegmentationMask, &relative_path, 4, 4)?;

    library.clear_generated_cache()?;

    assert!(data_root.join("masks").read_dir()?.next().is_none());
    assert_eq!(library.media_variants(item.id)?, Vec::new());
    assert!(library.managed_media_path(item.id)?.is_file());
    Ok(())
}
