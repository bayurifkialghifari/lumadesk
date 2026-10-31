use std::path::PathBuf;

use lumadesk_core::{
    CoreError, CropRect, EditRecipe, Library, MediaId, MediaItem, NewOverlay, OverlayId,
    OverlayInstance, SpriteSequence,
};
use tauri::{
    AppHandle, Emitter as _, LogicalPosition, LogicalSize, Manager as _, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

pub(crate) const OVERLAY_STATE_EVENT: &str = "overlay:state";
pub(crate) const OVERLAY_CATALOG_EVENT: &str = "overlay:catalog-changed";
pub(crate) const MEDIA_EDIT_RECIPE_EVENT: &str = "media:edit-recipe-changed";
pub(crate) const MEDIA_SPRITE_SEQUENCE_EVENT: &str = "media:sprite-sequence-changed";
const OVERLAY_LABEL_PREFIX: &str = "overlay-";
const MAIN_WINDOW_LABEL: &str = "main";
const FALLBACK_POSITION: f64 = 64.0;
const MAX_WINDOW_DIMENSION: f64 = 4_096.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct OverlaySize {
    width: f64,
    height: f64,
}

pub(crate) fn rendered_size(width: f64, source: &OverlayMedia) -> (f64, f64) {
    let size = overlay_size(width, source);
    (size.width, size.height)
}

#[derive(Clone, Debug)]
pub(crate) struct OverlayMedia {
    pub(crate) media: MediaItem,
    recipe: EditRecipe,
    sprite_sequence: Option<SpriteSequence>,
}

impl OverlayMedia {
    pub(crate) fn load(library: &Library, media_id: MediaId) -> Result<Self, CoreError> {
        Ok(Self {
            media: library.media_item(media_id)?,
            recipe: library.edit_recipe(media_id)?,
            sprite_sequence: library.sprite_sequence(media_id)?,
        })
    }
}

pub(crate) fn default_input(
    app: &AppHandle,
    source: &OverlayMedia,
    width: f64,
) -> tauri::Result<NewOverlay> {
    let size = overlay_size(width, source);
    let Some(monitor) = app.primary_monitor()? else {
        return Ok(NewOverlay {
            media_id: source.media.id,
            monitor_id: None,
            x: FALLBACK_POSITION,
            y: FALLBACK_POSITION,
        });
    };
    let scale_factor = monitor.scale_factor();
    let monitor_position = monitor.position().to_logical::<f64>(scale_factor);
    let monitor_size = monitor.size().to_logical::<f64>(scale_factor);
    Ok(NewOverlay {
        media_id: source.media.id,
        monitor_id: monitor.name().cloned(),
        x: centered_axis(monitor_position.x, monitor_size.width, size.width),
        y: centered_axis(monitor_position.y, monitor_size.height, size.height),
    })
}

pub(crate) fn sync(
    app: &AppHandle,
    overlay: &OverlayInstance,
    source: &OverlayMedia,
) -> tauri::Result<()> {
    if !overlay.is_active {
        return destroy(app, overlay.id);
    }

    let label = label(overlay.id);
    let size = overlay_size(overlay.width, source);
    let existing = app.get_webview_window(&label);
    let created = existing.is_none();
    let window = match existing {
        Some(window) => window,
        None => build(app, &label, overlay, size)?,
    };
    if let Err(error) = configure(&window, overlay, size)
        .and_then(|()| app.emit_to(label.as_str(), OVERLAY_STATE_EVENT, overlay.clone()))
    {
        if created && let Err(cleanup_error) = window.destroy() {
            eprintln!("failed to clean up overlay window {label}: {cleanup_error}");
        }
        return Err(error);
    }
    Ok(())
}

pub(crate) fn emit_state(app: &AppHandle, overlay: &OverlayInstance) -> tauri::Result<()> {
    app.emit_to(
        label(overlay.id).as_str(),
        OVERLAY_STATE_EVENT,
        overlay.clone(),
    )
}

pub(crate) fn emit_catalog_changed(app: &AppHandle) {
    if let Err(error) = app.emit_to(MAIN_WINDOW_LABEL, OVERLAY_CATALOG_EVENT, ()) {
        eprintln!("failed to notify main window about overlay changes: {error}");
    }
}

pub(crate) fn emit_edit_recipe_changed(app: &AppHandle, recipe: &EditRecipe) {
    if let Err(error) = app.emit(MEDIA_EDIT_RECIPE_EVENT, recipe) {
        eprintln!("failed to notify overlay windows about edit changes: {error}");
    }
    emit_catalog_changed(app);
}

pub(crate) fn emit_sprite_sequence_changed(app: &AppHandle, media_id: MediaId) {
    if let Err(error) = app.emit(MEDIA_SPRITE_SEQUENCE_EVENT, media_id) {
        eprintln!("failed to notify overlay windows about sprite changes: {error}");
    }
    emit_catalog_changed(app);
}

pub(crate) fn destroy(app: &AppHandle, id: OverlayId) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(&label(id)) {
        window.destroy()?;
    }
    Ok(())
}

pub(crate) fn label(id: OverlayId) -> String {
    format!("{OVERLAY_LABEL_PREFIX}{id}")
}

fn build(
    app: &AppHandle,
    label: &str,
    overlay: &OverlayInstance,
    size: OverlaySize,
) -> tauri::Result<WebviewWindow> {
    WebviewWindowBuilder::new(app, label, WebviewUrl::App(PathBuf::from("index.html")))
        .title("LumaDesk Overlay")
        .decorations(false)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .skip_taskbar(true)
        .shadow(false)
        .position(overlay.x, overlay.y)
        .inner_size(size.width, size.height)
        .resizable(false)
        .focused(false)
        .focusable(!overlay.click_through)
        .transparent(true)
        .visible(false)
        .build()
}

fn configure(
    window: &WebviewWindow,
    overlay: &OverlayInstance,
    size: OverlaySize,
) -> tauri::Result<()> {
    window.set_size(LogicalSize::new(size.width, size.height))?;
    window.set_position(LogicalPosition::new(overlay.x, overlay.y))?;
    window.set_always_on_top(true)?;
    window.set_focusable(!overlay.click_through)?;
    window.set_ignore_cursor_events(overlay.click_through)?;
    window.show()?;
    Ok(())
}

fn overlay_size(width: f64, source: &OverlayMedia) -> OverlaySize {
    let (media_width, media_height) = display_dimensions(source);
    size_for_dimensions(width, media_width, media_height)
}

fn size_for_dimensions(width: f64, media_width: f64, media_height: f64) -> OverlaySize {
    let width = width.clamp(1.0, MAX_WINDOW_DIMENSION);
    let height = width * media_height / media_width;
    if height <= MAX_WINDOW_DIMENSION {
        return OverlaySize {
            width,
            height: height.max(1.0),
        };
    }
    OverlaySize {
        width: (width * MAX_WINDOW_DIMENSION / height).max(1.0),
        height: MAX_WINDOW_DIMENSION,
    }
}

fn display_dimensions(source: &OverlayMedia) -> (f64, f64) {
    let (frame_width, frame_height) = source.sprite_sequence.as_ref().map_or(
        (source.media.width, source.media.height),
        |sequence| {
            (
                source.media.width / sequence.columns,
                source.media.height / sequence.rows,
            )
        },
    );
    edited_dimensions(
        frame_width,
        frame_height,
        source.recipe.crop,
        source.recipe.rotation_degrees,
    )
}

fn edited_dimensions(
    source_width: u32,
    source_height: u32,
    crop: Option<CropRect>,
    rotation_degrees: i16,
) -> (f64, f64) {
    let source_width = f64::from(source_width.max(1));
    let source_height = f64::from(source_height.max(1));
    let (width, height) = crop.map_or((source_width, source_height), |crop| {
        let x = crop.x.clamp(0.0, source_width - 1.0);
        let y = crop.y.clamp(0.0, source_height - 1.0);
        (
            crop.width.clamp(1.0, source_width - x),
            crop.height.clamp(1.0, source_height - y),
        )
    });
    if matches!(rotation_degrees, 90 | 270) {
        (height, width)
    } else {
        (width, height)
    }
}

fn centered_axis(origin: f64, available: f64, content: f64) -> f64 {
    origin + ((available - content) / 2.0).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_uses_stable_overlay_prefix() {
        let id = OverlayId::default();

        let result = label(id);

        assert_eq!(result, format!("overlay-{id}"));
    }

    #[test]
    fn overlay_size_preserves_landscape_aspect_ratio() {
        let (width, height) = edited_dimensions(1_920, 1_080, None, 0);
        let result = size_for_dimensions(320.0, width, height);

        assert_eq!(
            result,
            OverlaySize {
                width: 320.0,
                height: 180.0
            }
        );
    }

    #[test]
    fn overlay_size_bounds_extreme_portrait_media() {
        let result = size_for_dimensions(320.0, 1.0, 10_000.0);

        assert_eq!(result.height, MAX_WINDOW_DIMENSION);
        assert!(result.width >= 1.0);
    }

    #[test]
    fn edited_dimensions_clamp_crop_to_sprite_frame_then_rotate() {
        let result = edited_dimensions(
            100,
            50,
            Some(CropRect {
                x: 80.0,
                y: 10.0,
                width: 50.0,
                height: 30.0,
            }),
            90,
        );

        assert_eq!(result, (30.0, 20.0));
    }

    #[test]
    fn centered_axis_keeps_oversized_content_at_monitor_origin() {
        let result = centered_axis(-1_920.0, 1_920.0, 2_400.0);

        assert_eq!(result, -1_920.0);
    }
}
