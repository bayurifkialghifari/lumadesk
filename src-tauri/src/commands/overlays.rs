use lumadesk_core::{MediaId, MediaItem, NewOverlay, OverlayId, OverlayInstance, OverlayPatch};
use serde::Serialize;
use tauri::{AppHandle, Manager as _, State};

use crate::{
    command_error::CommandError,
    overlay_windows,
    state::{AppState, with_library},
};

const DUPLICATE_OFFSET: f64 = 24.0;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopOverlay {
    overlay: OverlayInstance,
    media: MediaItem,
}

/// Lists persisted desktop overlays.
///
/// # Errors
/// Returns a command error when overlay data cannot be read.
#[tauri::command]
pub(crate) async fn list_overlays(
    state: State<'_, AppState>,
) -> Result<Vec<OverlayInstance>, CommandError> {
    with_library(state, |library| library.list_overlays()).await
}

/// Lists active desktop overlays with their media metadata.
///
/// # Errors
/// Returns a command error when overlay or media data cannot be read.
#[tauri::command]
pub(crate) async fn list_desktop_overlays(
    state: State<'_, AppState>,
) -> Result<Vec<DesktopOverlay>, CommandError> {
    with_library(state, |library| {
        library
            .list_overlays()?
            .into_iter()
            .filter(|overlay| overlay.is_active)
            .map(|overlay| {
                let media = library.media_item(overlay.media_id)?;
                Ok(DesktopOverlay { overlay, media })
            })
            .collect()
    })
    .await
}

/// Creates a persisted desktop overlay.
///
/// # Errors
/// Returns a command error for invalid input, missing media, or unavailable storage.
#[tauri::command]
pub(crate) async fn create_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    input: NewOverlay,
) -> Result<OverlayInstance, CommandError> {
    let (overlay, source) = with_library(state, move |library| {
        let source = overlay_windows::OverlayMedia::load(library, input.media_id)?;
        let overlay = library.create_overlay(&input)?;
        Ok((overlay, source))
    })
    .await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &source) {
        rollback_created_overlay(&app, overlay.id).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(overlay)
}

/// Creates and centers a desktop overlay for one media item.
///
/// # Errors
/// Returns a command error when media is missing, monitor data is unavailable, persistence fails,
/// or the native window cannot be created.
#[tauri::command]
pub(crate) async fn show_media_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    id: MediaId,
) -> Result<OverlayInstance, CommandError> {
    let (source, width) = with_library(state, move |library| {
        let source = overlay_windows::OverlayMedia::load(library, id)?;
        let settings = library.get_settings()?;
        Ok((source, f64::from(settings.default_overlay_width)))
    })
    .await?;
    let input = overlay_windows::default_input(&app, &source, width)
        .map_err(|error| CommandError::overlay(&error))?;
    let overlay = with_library(app.state(), move |library| library.create_overlay(&input)).await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &source) {
        rollback_created_overlay(&app, overlay.id).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(overlay)
}

/// Applies validated changes to a desktop overlay.
///
/// # Errors
/// Returns a command error for invalid input, a missing overlay, or unavailable storage.
#[tauri::command]
pub(crate) async fn update_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    id: OverlayId,
    patch: OverlayPatch,
) -> Result<OverlayInstance, CommandError> {
    let (previous, overlay, source) = with_library(state, move |library| {
        let previous = library.overlay(id)?;
        let source = overlay_windows::OverlayMedia::load(library, previous.media_id)?;
        let overlay = library.update_overlay(id, &patch)?;
        Ok((previous, overlay, source))
    })
    .await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &source) {
        restore_updated_overlay(&app, previous, &source).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(overlay)
}

/// Persists the logical position reached by a native overlay drag.
///
/// # Errors
/// Returns a command error for invalid coordinates, a missing overlay, or unavailable storage.
#[tauri::command]
pub(crate) async fn move_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    id: OverlayId,
    monitor_id: Option<String>,
    x: f64,
    y: f64,
) -> Result<OverlayInstance, CommandError> {
    let patch = OverlayPatch {
        monitor_id: Some(monitor_id),
        x: Some(x),
        y: Some(y),
        ..OverlayPatch::default()
    };
    let (previous, overlay, source) = with_library(state, move |library| {
        let previous = library.overlay(id)?;
        let source = overlay_windows::OverlayMedia::load(library, previous.media_id)?;
        let overlay = library.update_overlay(id, &patch)?;
        Ok((previous, overlay, source))
    })
    .await?;
    if let Err(error) = overlay_windows::emit_state(&app, &overlay) {
        restore_updated_overlay(&app, previous, &source).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(overlay)
}

/// Copies an overlay and opens the new instance with a small position offset.
///
/// # Errors
/// Returns a command error when the source is missing, persistence fails, or the window cannot open.
#[tauri::command]
pub(crate) async fn duplicate_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    id: OverlayId,
) -> Result<OverlayInstance, CommandError> {
    let (overlay, source) = with_library(state, move |library| {
        let source = library.overlay(id)?;
        let media = overlay_windows::OverlayMedia::load(library, source.media_id)?;
        let overlay = library.duplicate_overlay(
            id,
            source.x + DUPLICATE_OFFSET,
            source.y + DUPLICATE_OFFSET,
        )?;
        Ok((overlay, media))
    })
    .await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &source) {
        rollback_created_overlay(&app, overlay.id).await;
        return Err(CommandError::overlay(&error));
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(overlay)
}

/// Removes a persisted desktop overlay.
///
/// # Errors
/// Returns a command error when the overlay is missing or cannot be removed.
#[tauri::command]
pub(crate) async fn remove_overlay(
    app: AppHandle,
    state: State<'_, AppState>,
    id: OverlayId,
) -> Result<(), CommandError> {
    let (overlay, source) = with_library(state, move |library| {
        let overlay = library.overlay(id)?;
        let source = overlay_windows::OverlayMedia::load(library, overlay.media_id)?;
        Ok((overlay, source))
    })
    .await?;
    overlay_windows::destroy(&app, id).map_err(|error| CommandError::overlay(&error))?;
    if let Err(error) = with_library(app.state(), move |library| library.remove_overlay(id)).await {
        if let Err(restore_error) = overlay_windows::sync(&app, &overlay, &source) {
            eprintln!("failed to restore overlay {id} after remove failed: {restore_error}");
        }
        return Err(error);
    }
    overlay_windows::emit_catalog_changed(&app);
    Ok(())
}

async fn rollback_created_overlay(app: &AppHandle, id: OverlayId) {
    let result = with_library(app.state(), move |library| library.remove_overlay(id)).await;
    if let Err(error) = result {
        eprintln!("failed to roll back overlay {id}: {error:?}");
    }
}

async fn restore_updated_overlay(
    app: &AppHandle,
    previous: OverlayInstance,
    source: &overlay_windows::OverlayMedia,
) {
    let id = previous.id;
    let patch = OverlayPatch {
        monitor_id: Some(previous.monitor_id.clone()),
        x: Some(previous.x),
        y: Some(previous.y),
        width: Some(previous.width),
        opacity: Some(previous.opacity),
        speed: Some(previous.speed),
        flip_horizontal: Some(previous.flip_horizontal),
        remove_background: Some(previous.remove_background),
        is_playing: Some(previous.is_playing),
        z_order: Some(previous.z_order),
        is_active: Some(previous.is_active),
    };
    let restored = with_library(app.state(), move |library| {
        library.update_overlay(id, &patch)
    })
    .await;
    let restored = match restored {
        Ok(restored) => restored,
        Err(error) => {
            eprintln!("failed to roll back overlay {id}: {error:?}");
            return;
        }
    };
    if let Err(error) = overlay_windows::sync(app, &restored, source) {
        eprintln!("failed to restore overlay window {id}: {error}");
    }
}
