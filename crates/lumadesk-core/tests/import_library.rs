use std::{error::Error, fs, io, path::Path};

use image::{
    Delay, DynamicImage, Frame, Rgba, RgbaImage,
    codecs::gif::{GifEncoder, Repeat},
};
use lumadesk_core::{
    AssetKind, BackgroundRemovalMode, CreatorChangesInput, CropRect, DesktopSceneItem,
    EditRecipePatch, ImportOutcome, Library, LibraryQuery, MediaFormat, MediaId, MediaItem,
    NewOverlay, OverlayBatchPatch, OverlayPatch, OverlayReplacement, SpritePlaybackMode,
    SpriteSequenceInput,
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
            "the first fixture must not be detected as a duplicate",
        )),
    }
}

#[test]
fn imports_static_and_animated_media_with_real_metadata() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let png = temp.path().join("biru.png");
    let gif = temp.path().join("blink.gif");
    write_png(&png)?;
    write_gif(&gif)?;
    let mut library = Library::open(temp.path().join("data"))?;

    let png_item = imported(library.import_file(&png)?)?;
    let gif_item = imported(library.import_file(&gif)?)?;

    assert_eq!((png_item.width, png_item.height), (4, 3));
    assert_eq!(png_item.asset_kind, AssetKind::Still);
    assert_eq!(png_item.media_format, MediaFormat::Png);
    assert_eq!(png_item.frame_count, 1);
    assert_eq!(gif_item.asset_kind, AssetKind::Animation);
    assert_eq!(gif_item.media_format, MediaFormat::Gif);
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
            return Err(
                io::Error::other("the second import must be detected as a duplicate").into(),
            );
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
    let broken = temp.path().join("broken.png");
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
            remove_background: Some(true),
            click_through: Some(true),
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
    assert_eq!(
        duplicate.remove_background,
        source_overlay.remove_background
    );
    assert_eq!(duplicate.is_playing, source_overlay.is_playing);
    assert_eq!(duplicate.click_through, source_overlay.click_through);
    assert_eq!(duplicate.z_order, 5);
    assert!(duplicate.is_active);
    assert_eq!(library.list_overlays()?.len(), 2);
    Ok(())
}

fn scene_item(media_id: MediaId) -> DesktopSceneItem {
    DesktopSceneItem {
        media_id,
        monitor_name: Some("Primary".into()),
        monitor_width: 1_920.0,
        monitor_height: 1_080.0,
        normalized_x: 0.25,
        normalized_y: 0.5,
        width: 320.0,
        opacity: 0.8,
        speed: 1.5,
        flip_horizontal: true,
        remove_background: true,
        is_playing: false,
        click_through: true,
        z_order: 3,
        is_active: false,
    }
}

#[test]
fn desktop_scene_lifecycle_uses_explicit_snapshots_and_unique_names() -> Result<(), Box<dyn Error>>
{
    let temp = TempDir::new()?;
    let source = temp.path().join("scene.png");
    write_png(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let media = imported(library.import_file(&source)?)?;
    let original_item = scene_item(media.id);

    let created =
        library.create_desktop_scene("  Focus  ", std::slice::from_ref(&original_item))?;
    assert_eq!(created.name, "Focus");
    assert_eq!(
        created.items.as_slice(),
        std::slice::from_ref(&original_item)
    );
    assert!(library.create_desktop_scene("focus", &[]).is_err());
    assert!(library.create_desktop_scene("", &[]).is_err());

    let overlay = library.create_overlay(&NewOverlay {
        media_id: media.id,
        monitor_id: None,
        x: 1.0,
        y: 2.0,
    })?;
    library.update_overlay(
        overlay.id,
        &OverlayPatch {
            width: Some(700.0),
            ..OverlayPatch::default()
        },
    )?;
    assert_eq!(library.desktop_scene(created.id)?.items, [original_item]);

    let updated_item = DesktopSceneItem {
        normalized_x: 0.75,
        ..scene_item(media.id)
    };
    let updated = library.update_desktop_scene(created.id, std::slice::from_ref(&updated_item))?;
    assert_eq!(updated.items, [updated_item]);
    let renamed = library.rename_desktop_scene(created.id, "Presentation")?;
    assert_eq!(renamed.name, "Presentation");
    let duplicate = library.duplicate_desktop_scene(created.id)?;
    assert_eq!(duplicate.name, "Presentation copy");
    assert_eq!(duplicate.items, renamed.items);
    assert_eq!(library.list_desktop_scenes()?.len(), 2);

    library.delete_desktop_scene(created.id)?;
    assert!(library.desktop_scene(created.id).is_err());
    Ok(())
}

#[test]
fn deleting_media_cascades_scene_items_but_keeps_the_scene() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("scene-cascade.png");
    write_png(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let media = imported(library.import_file(&source)?)?;
    let scene = library.create_desktop_scene("Cascade", &[scene_item(media.id)])?;

    library.delete_media(media.id)?;

    assert!(library.desktop_scene(scene.id)?.items.is_empty());
    assert_eq!(library.list_desktop_scenes()?[0].item_count, 0);
    Ok(())
}

#[test]
fn replacing_and_batch_patching_overlays_are_atomic() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("replacement.png");
    write_png(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let media = imported(library.import_file(&source)?)?;
    let existing = library.create_overlay(&NewOverlay {
        media_id: media.id,
        monitor_id: None,
        x: 10.0,
        y: 20.0,
    })?;
    let missing = MediaId::new();
    let replacement = OverlayReplacement {
        media_id: missing,
        monitor_id: None,
        x: 0.0,
        y: 0.0,
        width: 320.0,
        opacity: 1.0,
        speed: 1.0,
        flip_horizontal: false,
        remove_background: false,
        is_playing: true,
        click_through: false,
        z_order: 0,
        is_active: true,
    };
    assert!(library.replace_overlays(&[replacement]).is_err());
    assert_eq!(
        library.list_overlays()?.as_slice(),
        std::slice::from_ref(&existing)
    );

    let patched = library.patch_all_overlays(&OverlayBatchPatch {
        is_playing: Some(false),
        is_active: Some(false),
        click_through: Some(true),
    })?;
    assert_eq!(patched.len(), 1);
    assert!(!patched[0].is_playing);
    assert!(!patched[0].is_active);
    assert!(patched[0].click_through);

    assert!(library.replace_overlays(&[])?.is_empty());
    assert!(library.list_overlays()?.is_empty());
    Ok(())
}

#[test]
fn catalog_edits_preserve_managed_file_and_drive_filters() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("original-name.png");
    write_png(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let original = imported(library.import_file(&source)?)?;
    let managed_path = library.managed_media_path(original.id)?;

    let renamed = library.rename_media(original.id, "Blue Light")?;
    assert_eq!(renamed.display_name, "Blue Light");
    assert_eq!(library.managed_media_path(original.id)?, managed_path);
    assert!(source.exists());

    let tagged = library.replace_tags(
        original.id,
        &["Pixel".into(), " ambient ".into(), "pixel".into()],
    )?;
    assert_eq!(tagged.tags, ["ambient", "Pixel"]);
    assert!(library.set_favorite(original.id, true)?.is_favorite);

    let by_tag = LibraryQuery {
        search: Some("ambient".into()),
        ..LibraryQuery::default()
    };
    assert_eq!(library.list_media(&by_tag)?.total, 1);
    let favorites = LibraryQuery {
        media_format: Some(MediaFormat::Png),
        favorite_only: true,
        ..LibraryQuery::default()
    };
    assert_eq!(library.list_media(&favorites)?.total, 1);
    Ok(())
}

#[test]
fn edit_recipes_and_sprite_metadata_preserve_imported_source() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("sheet.png");
    write_png(&source)?;
    let source_bytes = fs::read(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let item = imported(library.import_file(&source)?)?;
    let managed_path = library.managed_media_path(item.id)?;
    let managed_bytes = fs::read(&managed_path)?;

    let recipe = library.update_edit_recipe(
        item.id,
        &EditRecipePatch {
            crop: Some(Some(CropRect {
                x: 0.0,
                y: 0.0,
                width: 2.0,
                height: 1.0,
            })),
            rotation_degrees: Some(90),
            flip_horizontal: Some(true),
            background_removal: Some(BackgroundRemovalMode::Edge),
            brightness: Some(1.2),
            contrast: Some(0.8),
            saturation: Some(1.4),
        },
    )?;
    assert_eq!(library.edit_recipe(item.id)?, recipe);

    let sequence = library.set_sprite_sequence(&SpriteSequenceInput {
        media_id: item.id,
        columns: 2,
        rows: 2,
        frame_count: 4,
        frame_duration_ms: 125,
        playback_mode: SpritePlaybackMode::PingPong,
    })?;
    assert_eq!(library.sprite_sequence(item.id)?, Some(sequence));
    let sprite_item = library.media_item(item.id)?;
    assert_eq!(sprite_item.asset_kind, AssetKind::SpriteSheet);
    assert_eq!(sprite_item.frame_count, 4);
    assert_eq!(sprite_item.duration_ms, 500);

    assert_eq!(fs::read(&source)?, source_bytes);
    assert_eq!(fs::read(&managed_path)?, managed_bytes);
    assert_eq!(library.media_variants(item.id)?, Vec::new());

    library.clear_sprite_sequence(item.id)?;
    assert_eq!(library.sprite_sequence(item.id)?, None);
    assert_eq!(library.media_item(item.id)?.asset_kind, AssetKind::Still);
    Ok(())
}

#[test]
fn creator_save_validates_before_committing_recipe_and_sprite() -> Result<(), Box<dyn Error>> {
    let temp = TempDir::new()?;
    let source = temp.path().join("creator-sheet.png");
    write_png(&source)?;
    let mut library = Library::open(temp.path().join("data"))?;
    let item = imported(library.import_file(&source)?)?;
    let original_recipe = library.edit_recipe(item.id)?;

    let invalid = library.save_creator_changes(&CreatorChangesInput {
        media_id: item.id,
        recipe: EditRecipePatch {
            crop: None,
            rotation_degrees: None,
            flip_horizontal: None,
            background_removal: None,
            brightness: Some(1.5),
            contrast: None,
            saturation: None,
        },
        sprite_sequence: Some(SpriteSequenceInput {
            media_id: item.id,
            columns: 0,
            rows: 1,
            frame_count: 1,
            frame_duration_ms: 100,
            playback_mode: SpritePlaybackMode::Loop,
        }),
        clear_sprite_sequence: false,
    });

    assert!(invalid.is_err());
    assert_eq!(library.edit_recipe(item.id)?, original_recipe);
    assert_eq!(library.sprite_sequence(item.id)?, None);

    let saved = library.save_creator_changes(&CreatorChangesInput {
        media_id: item.id,
        recipe: EditRecipePatch {
            crop: Some(Some(CropRect {
                x: 0.0,
                y: 0.0,
                width: 2.0,
                height: 2.0,
            })),
            rotation_degrees: Some(90),
            flip_horizontal: Some(true),
            background_removal: Some(BackgroundRemovalMode::Edge),
            brightness: Some(1.2),
            contrast: Some(0.8),
            saturation: Some(1.4),
        },
        sprite_sequence: Some(SpriteSequenceInput {
            media_id: item.id,
            columns: 2,
            rows: 1,
            frame_count: 2,
            frame_duration_ms: 120,
            playback_mode: SpritePlaybackMode::PingPong,
        }),
        clear_sprite_sequence: false,
    })?;

    assert!((saved.recipe.brightness - 1.2).abs() <= f64::EPSILON);
    assert_eq!(
        saved
            .sprite_sequence
            .as_ref()
            .map(|value| value.frame_count),
        Some(2)
    );
    assert_eq!(library.edit_recipe(item.id)?, saved.recipe);
    assert_eq!(library.sprite_sequence(item.id)?, saved.sprite_sequence);
    Ok(())
}
