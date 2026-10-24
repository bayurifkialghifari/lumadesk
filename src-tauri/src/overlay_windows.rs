use std::path::PathBuf;

use lumadesk_core::{MediaItem, NewOverlay, OverlayId, OverlayInstance};
use tauri::{
    AppHandle, Emitter as _, LogicalPosition, LogicalSize, Manager as _, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

pub(crate) const OVERLAY_STATE_EVENT: &str = "overlay:state";
const OVERLAY_LABEL_PREFIX: &str = "overlay-";
const FALLBACK_POSITION: f64 = 64.0;
const MAX_WINDOW_DIMENSION: f64 = 4_096.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct OverlaySize {
    width: f64,
    height: f64,
}

pub(crate) fn default_input(
    app: &AppHandle,
    media: &MediaItem,
    width: f64,
) -> tauri::Result<NewOverlay> {
    let size = overlay_size(width, media.width, media.height);
    let Some(monitor) = app.primary_monitor()? else {
        return Ok(NewOverlay {
            media_id: media.id,
            monitor_id: None,
            x: FALLBACK_POSITION,
            y: FALLBACK_POSITION,
        });
    };
    let scale_factor = monitor.scale_factor();
    let monitor_position = monitor.position().to_logical::<f64>(scale_factor);
    let monitor_size = monitor.size().to_logical::<f64>(scale_factor);
    Ok(NewOverlay {
        media_id: media.id,
        monitor_id: monitor.name().cloned(),
        x: centered_axis(monitor_position.x, monitor_size.width, size.width),
        y: centered_axis(monitor_position.y, monitor_size.height, size.height),
    })
}

pub(crate) fn sync(
    app: &AppHandle,
    overlay: &OverlayInstance,
    media: &MediaItem,
) -> tauri::Result<()> {
    if !overlay.is_active {
        return destroy(app, overlay.id);
    }

    let label = label(overlay.id);
    let size = overlay_size(overlay.width, media.width, media.height);
    let existing = app.get_webview_window(&label);
    let created = existing.is_none();
    let window = match existing {
        Some(window) => window,
        None => build(app, &label, overlay, size)?,
    };
    if let Err(error) = configure(&window, overlay, size)
        .and_then(|()| app.emit_to(label.as_str(), OVERLAY_STATE_EVENT, overlay.clone()))
    {
        if created {
            if let Err(cleanup_error) = window.destroy() {
                eprintln!("failed to clean up overlay window {label}: {cleanup_error}");
            }
        }
        return Err(error);
    }
    Ok(())
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
        .focusable(false)
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
    window.set_ignore_cursor_events(true)?;
    window.show()?;
    Ok(())
}

fn overlay_size(width: f64, media_width: u32, media_height: u32) -> OverlaySize {
    let width = width.clamp(1.0, MAX_WINDOW_DIMENSION);
    let height = width * f64::from(media_height) / f64::from(media_width);
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
        let result = overlay_size(320.0, 1_920, 1_080);

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
        let result = overlay_size(320.0, 1, 10_000);

        assert_eq!(result.height, MAX_WINDOW_DIMENSION);
        assert!(result.width >= 1.0);
    }

    #[test]
    fn centered_axis_keeps_oversized_content_at_monitor_origin() {
        let result = centered_axis(-1_920.0, 1_920.0, 2_400.0);

        assert_eq!(result, -1_920.0);
    }
}
