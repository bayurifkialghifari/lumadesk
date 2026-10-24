use std::path::PathBuf;

use lumadesk_core::{
    AppSettings, DeleteReport, ImportReport, LibraryQuery, MediaId, MediaItem, MediaPage,
    NewOverlay, OverlayId, OverlayInstance, OverlayPatch,
};
use tauri::State;

use crate::{
    command_error::CommandError,
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
    state: State<'_, AppState>,
    id: MediaId,
) -> Result<DeleteReport, CommandError> {
    with_library(state, move |library| library.delete_media(id)).await
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
    state: State<'_, AppState>,
    input: NewOverlay,
) -> Result<OverlayInstance, CommandError> {
    with_library(state, move |library| library.create_overlay(&input)).await
}

/// Applies validated changes to a desktop overlay.
///
/// # Errors
/// Returns a command error for invalid input, a missing overlay, or unavailable storage.
#[tauri::command]
pub(crate) async fn update_overlay(
    state: State<'_, AppState>,
    id: OverlayId,
    patch: OverlayPatch,
) -> Result<OverlayInstance, CommandError> {
    with_library(state, move |library| library.update_overlay(id, &patch)).await
}

/// Removes a persisted desktop overlay.
///
/// # Errors
/// Returns a command error when the overlay is missing or cannot be removed.
#[tauri::command]
pub(crate) async fn remove_overlay(
    state: State<'_, AppState>,
    id: OverlayId,
) -> Result<(), CommandError> {
    with_library(state, move |library| library.remove_overlay(id)).await
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
