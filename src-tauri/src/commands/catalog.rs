use std::path::PathBuf;

use lumadesk_core::{
    DeleteReport, FrameBatch, FrameBatchRequest, ImportReport, LibraryQuery, MediaId, MediaItem,
    MediaPage, OverlayInstance,
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
        Ok(report) => {
            overlay_windows::emit_catalog_changed(&app);
            Ok(report)
        }
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
