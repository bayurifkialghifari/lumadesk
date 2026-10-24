use std::path::PathBuf;

use lumadesk_core::{
    AppSettings, DeleteReport, FrameBatch, FrameBatchRequest, ImportReport, LibraryQuery, MediaId,
    MediaItem, MediaPage, NewOverlay, OverlayId, OverlayInstance, OverlayPatch,
};
use tauri::{AppHandle, Manager as _, State};

use crate::{
    command_error::CommandError,
    overlay_windows,
    state::{AppState, with_library},
};

/// Lists media using the supplied catalog query.
///
/// # Errors
/// Returns a serializable command error when local data cannot be read.
#[tauri::command]
pub(crate) async fn list_media(
    state: State<'_, AppState>,
    query: LibraryQuery,
) -> Result<MediaPage, CommandError> {
    with_library(state, move |library| library.list_media(&query)).await
}

/// Imports local files into the managed media store.
///
/// # Errors
/// Returns a command error only when the import worker itself cannot run.
#[tauri::command]
pub(crate) async fn import_media(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<ImportReport, CommandError> {
    with_library(state, move |library| {
        let paths = paths.into_iter().map(PathBuf::from);
        Ok(library.import_paths(paths))
    })
    .await
}

/// Changes an item's display name.
///
/// # Errors
/// Returns a command error for invalid input, missing media, or unavailable storage.
#[tauri::command]
pub(crate) async fn rename_media(
    state: State<'_, AppState>,
    id: MediaId,
    name: String,
) -> Result<MediaItem, CommandError> {
    with_library(state, move |library| library.rename_media(id, &name)).await
}

/// Changes an item's favorite state.
///
/// # Errors
/// Returns a command error when the item is missing or storage cannot be updated.
#[tauri::command]
pub(crate) async fn set_favorite(
    state: State<'_, AppState>,
    id: MediaId,
    favorite: bool,
) -> Result<MediaItem, CommandError> {
    with_library(state, move |library| library.set_favorite(id, favorite)).await
}

/// Replaces all tags attached to a media item.
///
/// # Errors
/// Returns a command error for invalid tags, missing media, or unavailable storage.
#[tauri::command]
pub(crate) async fn replace_tags(
    state: State<'_, AppState>,
    id: MediaId,
    tags: Vec<String>,
) -> Result<MediaItem, CommandError> {
    with_library(state, move |library| library.replace_tags(id, &tags)).await
}

/// Deletes media and its dependent managed data.
///
/// # Errors
/// Returns a command error when the item or its managed files cannot be removed safely.
#[tauri::command]
pub(crate) async fn delete_media(
    app: AppHandle,
    state: State<'_, AppState>,
    id: MediaId,
) -> Result<DeleteReport, CommandError> {
    let (media, overlays) = with_library(state, move |library| {
        let media = library.media_item(id)?;
        let overlays = library
            .list_overlays()?
            .into_iter()
            .filter(|overlay| overlay.media_id == id)
            .collect::<Vec<_>>();
        Ok((media, overlays))
    })
    .await?;
    for (index, overlay) in overlays.iter().enumerate() {
        if let Err(error) = overlay_windows::destroy(&app, overlay.id) {
            restore_overlay_windows(&app, &overlays[..index], &media);
            return Err(CommandError::overlay(&error));
        }
    }
    match with_library(app.state(), move |library| library.delete_media(id)).await {
        Ok(report) => Ok(report),
        Err(error) => {
            restore_overlay_windows(&app, &overlays, &media);
            Err(error)
        }
    }
}

/// Resolves a managed media identifier to a path accepted by the asset protocol.
///
/// # Errors
/// Returns a command error when the item or its managed path is unavailable.
#[tauri::command]
pub(crate) async fn resolve_media_path(
    state: State<'_, AppState>,
    id: MediaId,
) -> Result<String, CommandError> {
    with_library(state, move |library| {
        Ok(library
            .managed_media_path(id)?
            .to_string_lossy()
            .into_owned())
    })
    .await
}

/// Returns a bounded batch of decoded frame assets from the local cache.
///
/// # Errors
/// Returns a command error when media is missing, decoding fails, or the batch is invalid.
#[tauri::command]
pub(crate) async fn get_frame_batch(
    state: State<'_, AppState>,
    id: MediaId,
    offset: u32,
    limit: u32,
) -> Result<FrameBatch, CommandError> {
    with_library(state, move |library| {
        library.frame_batch(id, FrameBatchRequest { offset, limit })
    })
    .await
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
    let (overlay, media) = with_library(state, move |library| {
        let media = library.media_item(input.media_id)?;
        let overlay = library.create_overlay(&input)?;
        Ok((overlay, media))
    })
    .await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &media) {
        rollback_created_overlay(&app, overlay.id).await;
        return Err(CommandError::overlay(&error));
    }
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
    let (media, width) = with_library(state, move |library| {
        let media = library.media_item(id)?;
        let settings = library.get_settings()?;
        Ok((media, f64::from(settings.default_overlay_width)))
    })
    .await?;
    let input = overlay_windows::default_input(&app, &media, width)
        .map_err(|error| CommandError::overlay(&error))?;
    let overlay = with_library(app.state(), move |library| library.create_overlay(&input)).await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &media) {
        rollback_created_overlay(&app, overlay.id).await;
        return Err(CommandError::overlay(&error));
    }
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
    let (previous, overlay, media) = with_library(state, move |library| {
        let previous = library.overlay(id)?;
        let media = library.media_item(previous.media_id)?;
        let overlay = library.update_overlay(id, &patch)?;
        Ok((previous, overlay, media))
    })
    .await?;
    if let Err(error) = overlay_windows::sync(&app, &overlay, &media) {
        restore_updated_overlay(&app, previous, &media).await;
        return Err(CommandError::overlay(&error));
    }
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
    let (overlay, media) = with_library(state, move |library| {
        let overlay = library.overlay(id)?;
        let media = library.media_item(overlay.media_id)?;
        Ok((overlay, media))
    })
    .await?;
    overlay_windows::destroy(&app, id).map_err(|error| CommandError::overlay(&error))?;
    if let Err(error) = with_library(app.state(), move |library| library.remove_overlay(id)).await {
        if let Err(restore_error) = overlay_windows::sync(&app, &overlay, &media) {
            eprintln!("failed to restore overlay {id} after remove failed: {restore_error}");
        }
        return Err(error);
    }
    Ok(())
}

/// Reads typed application settings.
///
/// # Errors
/// Returns a command error when local settings cannot be read.
#[tauri::command]
pub(crate) async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    with_library(state, |library| library.get_settings()).await
}

/// Validates and persists application settings.
///
/// # Errors
/// Returns a command error for invalid values or unavailable storage.
#[tauri::command]
pub(crate) async fn update_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, CommandError> {
    with_library(state, move |library| library.update_settings(&settings)).await
}

async fn rollback_created_overlay(app: &AppHandle, id: OverlayId) {
    let result = with_library(app.state(), move |library| library.remove_overlay(id)).await;
    if let Err(error) = result {
        eprintln!("failed to roll back overlay {id}: {error:?}");
    }
}

fn restore_overlay_windows(app: &AppHandle, overlays: &[OverlayInstance], media: &MediaItem) {
    for overlay in overlays {
        if let Err(error) = overlay_windows::sync(app, overlay, media) {
            eprintln!(
                "failed to restore overlay {} after delete failed: {error}",
                overlay.id
            );
        }
    }
}

async fn restore_updated_overlay(app: &AppHandle, previous: OverlayInstance, media: &MediaItem) {
    let id = previous.id;
    let patch = OverlayPatch {
        monitor_id: Some(previous.monitor_id.clone()),
        x: Some(previous.x),
        y: Some(previous.y),
        width: Some(previous.width),
        opacity: Some(previous.opacity),
        speed: Some(previous.speed),
        flip_horizontal: Some(previous.flip_horizontal),
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
    if let Err(error) = overlay_windows::sync(app, &restored, media) {
        eprintln!("failed to restore overlay window {id}: {error}");
    }
}
