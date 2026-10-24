use std::{error::Error, fs, io, path::Path};

use image::{
    Delay, DynamicImage, Frame, Rgba, RgbaImage,
    codecs::gif::{GifEncoder, Repeat},
};
use lumadesk_core::{
    ImportOutcome, Library, LibraryQuery, MediaItem, MediaKind, NewOverlay, OverlayPatch,
};
use tempfile::TempDir;

fn write_png(path: &Path) -> Result<(), image::ImageError> {
    let pixels = RgbaImage::from_pixel(4, 3, Rgba([31, 168, 255, 255]));
    DynamicImage::ImageRgba8(pixels).save(path)
}

fn write_gif(path: &Path) -> Result<(), Box<dyn Error>> {
    let file = fs::File::create(path)?;
    let mut encoder = GifEncoder::new(file);
    encoder.set_repeat(Repeat::Infinite)?;
    for color in [[255, 0, 0, 255], [0, 0, 255, 255]] {
        let pixels = RgbaImage::from_pixel(2, 2, Rgba(color));
        let frame = Frame::from_parts(pixels, 0, 0, Delay::from_numer_denom_ms(80, 1));
        encoder.encode_frame(frame)?;
    }
    Ok(())
}

fn imported(outcome: ImportOutcome) -> Result<MediaItem, io::Error> {
    match outcome {
        ImportOutcome::Imported { item } => Ok(item),
        ImportOutcome::Duplicate { .. } => Err(io::Error::other(
            "fixture pertama tidak boleh terdeteksi sebagai duplikat",
        )),
    }
}

#[test]
fn imports_static_and_animated_media_with_real_metadata() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let png = temp.path().join("biru.png");
    let gif = temp.path().join("kedip.gif");
    write_png(&png)?;
    write_gif(&gif)?;
    let mut library = Library::open(temp.path().join("data"))?;

    let png_item = imported(library.import_file(&png)?)?;
    let gif_item = imported(library.import_file(&gif)?)?;

    assert_eq!((png_item.width, png_item.height), (4, 3));
    assert_eq!(png_item.media_kind, MediaKind::Png);
    assert_eq!(png_item.frame_count, 1);
    assert_eq!(gif_item.media_kind, MediaKind::Gif);
    assert_eq!(gif_item.frame_count, 2);
    assert_eq!(gif_item.duration_ms, 160);
    assert!(library.managed_media_path(gif_item.id)?.exists());
    Ok(())
}

#[test]
fn duplicate_import_reuses_catalog_item_and_managed_file() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let first = temp.path().join("first.png");
    let second = temp.path().join("second.png");
    write_png(&first)?;
    fs::copy(&first, &second)?;
    let data = temp.path().join("data");
    let mut library = Library::open(&data)?;
    let original = imported(library.import_file(&first)?)?;

    let duplicate = library.import_file(&second)?;
    let item = match duplicate {
        ImportOutcome::Duplicate { item } => item,
        ImportOutcome::Imported { .. } => {
            return Err(io::Error::other("impor kedua harus terdeteksi sebagai duplikat").into());
        }
    };
    assert_eq!(item.id, original.id);
    assert_eq!(library.list_media(&LibraryQuery::default())?.total, 1);
    assert_eq!(fs::read_dir(data.join("media"))?.count(), 1);
    Ok(())
}

#[test]
fn rejected_media_leaves_no_row_or_partial_file() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let broken = temp.path().join("rusak.png");
    fs::write(&broken, b"not a real image")?;
    let data = temp.path().join("data");
    let mut library = Library::open(&data)?;

    assert!(library.import_file(&broken).is_err());
    assert_eq!(library.list_media(&LibraryQuery::default())?.total, 0);
    assert_eq!(fs::read_dir(data.join("media"))?.count(), 0);
    Ok(())
}

#[test]
fn deleting_media_cascades_overlays_without_touching_source() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source.png");
    write_png(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let item = imported(library.import_file(&source)?)?;
    let overlay = library.create_overlay(&NewOverlay {
        media_id: item.id,
        monitor_id: None,
        x: 20.0,
        y: 30.0,
    })?;
    let updated = library.update_overlay(
        overlay.id,
        &OverlayPatch {
            speed: Some(2.0),
            ..OverlayPatch::default()
        },
    )?;
    assert!((updated.speed - 2.0).abs() <= f64::EPSILON);
    assert_eq!(library.overlay(overlay.id)?, updated);

    let report = library.delete_media(item.id)?;
    assert_eq!(report.overlays_removed, 1);
    assert!(source.exists());
    assert_eq!(library.list_overlays()?, Vec::new());
    Ok(())
}

#[test]
fn duplicated_overlay_copies_controls_at_the_requested_position() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source.png");
    write_png(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let item = imported(library.import_file(&source)?)?;
    let overlay = library.create_overlay(&NewOverlay {
        media_id: item.id,
        monitor_id: Some("Primary".into()),
        x: 20.0,
        y: 30.0,
    })?;
    let source_overlay = library.update_overlay(
        overlay.id,
        &OverlayPatch {
            width: Some(480.0),
            opacity: Some(0.7),
            speed: Some(1.5),
            flip_horizontal: Some(true),
            is_playing: Some(false),
            z_order: Some(4),
            ..OverlayPatch::default()
        },
    )?;

    let duplicate = library.duplicate_overlay(source_overlay.id, 44.0, 54.0)?;

    assert_ne!(duplicate.id, source_overlay.id);
    assert_eq!(duplicate.media_id, source_overlay.media_id);
    assert_eq!(duplicate.monitor_id, source_overlay.monitor_id);
    assert_eq!((duplicate.x, duplicate.y), (44.0, 54.0));
    assert!((duplicate.width - source_overlay.width).abs() < f64::EPSILON);
    assert!((duplicate.opacity - source_overlay.opacity).abs() < f64::EPSILON);
    assert!((duplicate.speed - source_overlay.speed).abs() < f64::EPSILON);
    assert_eq!(duplicate.flip_horizontal, source_overlay.flip_horizontal);
    assert_eq!(duplicate.is_playing, source_overlay.is_playing);
    assert_eq!(duplicate.z_order, 5);
    assert!(duplicate.is_active);
    assert_eq!(library.list_overlays()?.len(), 2);
    Ok(())
}

#[test]
fn catalog_edits_preserve_managed_file_and_drive_filters() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("nama-asli.png");
    write_png(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let original = imported(library.import_file(&source)?)?;
    let managed_path = library.managed_media_path(original.id)?;

    let renamed = library.rename_media(original.id, "Lampu Biru")?;
    assert_eq!(renamed.display_name, "Lampu Biru");
    assert_eq!(library.managed_media_path(original.id)?, managed_path);
    assert!(source.exists());

    let tagged = library.replace_tags(
        original.id,
        &["Pixel".into(), " santai ".into(), "pixel".into()],
    )?;
    assert_eq!(tagged.tags, ["Pixel", "santai"]);
    assert!(library.set_favorite(original.id, true)?.is_favorite);

    let by_tag = LibraryQuery {
        search: Some("santai".into()),
        ..LibraryQuery::default()
    };
    assert_eq!(library.list_media(&by_tag)?.total, 1);
    let favorites = LibraryQuery {
        media_kind: Some(MediaKind::Png),
        favorite_only: true,
        ..LibraryQuery::default()
    };
    assert_eq!(library.list_media(&favorites)?.total, 1);
    Ok(())
}
