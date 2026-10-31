use std::path::PathBuf;

use lumadesk_core::{
    CreatorChanges, CreatorChangesInput, DeleteReport, EditRecipe, EditRecipePatch, FrameBatch,
    FrameBatchRequest, ImportReport, LibraryQuery, MediaId, MediaItem, MediaPage, MediaVariant,
    OverlayInstance, SpriteSequence, SpriteSequenceInput,
};
use tauri::{AppHandle, Manager as _, State};

use crate::{
    ai::AiState,
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
    ai_state: State<'_, AiState>,
    state: State<'_, AppState>,
    id: MediaId,
) -> Result<DeleteReport, CommandError> {
    let _generated_media = ai_state.lock_generated_media().await;
    let (source, overlays) = with_library(state, move |library| {
        let source = overlay_windows::OverlayMedia::load(library, id)?;
        let overlays = library
            .list_overlays()?
            .into_iter()
            .filter(|overlay| overlay.media_id == id)
            .collect::<Vec<_>>();
        Ok((source, overlays))
    })
    .await?;
    for (index, overlay) in overlays.iter().enumerate() {
        if let Err(error) = overlay_windows::destroy(&app, overlay.id) {
            restore_overlay_windows(&app, &overlays[..index], &source);
            return Err(CommandError::overlay(&error));
        }
    }
    match with_library(app.state(), move |library| library.delete_media(id)).await {
        Ok(report) => {
            overlay_windows::emit_catalog_changed(&app);
            Ok(report)
        }
        Err(error) => {
            restore_overlay_windows(&app, &overlays, &source);
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

/// Returns the non-destructive edit recipe for one media item.
///
/// # Errors
/// Returns a command error when the item or recipe cannot be read.
#[tauri::command]
pub(crate) async fn get_edit_recipe(
    state: State<'_, AppState>,
    id: MediaId,
) -> Result<EditRecipe, CommandError> {
    with_library(state, move |library| library.edit_recipe(id)).await
}

/// Updates a non-destructive edit recipe.
///
/// # Errors
/// Returns a command error when the patch is invalid or cannot be stored.
#[tauri::command]
pub(crate) async fn update_edit_recipe(
    app: AppHandle,
    state: State<'_, AppState>,
    id: MediaId,
    patch: EditRecipePatch,
) -> Result<EditRecipe, CommandError> {
    let (recipe, source, overlays) = with_library(state, move |library| {
        let recipe = library.update_edit_recipe(id, &patch)?;
        let source = overlay_windows::OverlayMedia::load(library, id)?;
        let overlays = active_overlays_for(library, id)?;
        Ok((recipe, source, overlays))
    })
    .await?;
    sync_media_windows(&app, &overlays, &source);
    overlay_windows::emit_edit_recipe_changed(&app, &recipe);
    Ok(recipe)
}

/// Saves the edit recipe and sprite configuration as one Creator Studio operation.
///
/// # Errors
/// Returns a command error when validation, persistence, or generated-cache cleanup fails.
#[tauri::command]
pub(crate) async fn save_creator_changes(
    app: AppHandle,
    ai_state: State<'_, AiState>,
    state: State<'_, AppState>,
    input: CreatorChangesInput,
) -> Result<CreatorChanges, CommandError> {
    let _generated_media = ai_state.lock_generated_media().await;
    let media_id = input.media_id;
    let (changes, source, overlays) = with_library(state, move |library| {
        let changes = library.save_creator_changes(&input)?;
        let source = overlay_windows::OverlayMedia::load(library, media_id)?;
        let overlays = active_overlays_for(library, media_id)?;
        Ok((changes, source, overlays))
    })
    .await?;
    sync_media_windows(&app, &overlays, &source);
    overlay_windows::emit_edit_recipe_changed(&app, &changes.recipe);
    overlay_windows::emit_sprite_sequence_changed(&app, media_id);
    Ok(changes)
}

/// Returns sprite-sheet playback metadata when configured.
///
/// # Errors
/// Returns a command error when the item or metadata cannot be read.
#[tauri::command]
pub(crate) async fn get_sprite_sequence(
    state: State<'_, AppState>,
    id: MediaId,
) -> Result<Option<SpriteSequence>, CommandError> {
    with_library(state, move |library| library.sprite_sequence(id)).await
}

/// Creates or replaces sprite-sheet playback metadata.
///
/// # Errors
/// Returns a command error when the grid is invalid or cannot be stored.
#[tauri::command]
pub(crate) async fn set_sprite_sequence(
    app: AppHandle,
    ai_state: State<'_, AiState>,
    state: State<'_, AppState>,
    input: SpriteSequenceInput,
) -> Result<SpriteSequence, CommandError> {
    let _generated_media = ai_state.lock_generated_media().await;
    let media_id = input.media_id;
    let (sequence, source, overlays) = with_library(state, move |library| {
        let sequence = library.set_sprite_sequence(&input)?;
        let source = overlay_windows::OverlayMedia::load(library, media_id)?;
        let overlays = active_overlays_for(library, media_id)?;
        Ok((sequence, source, overlays))
    })
    .await?;
    sync_media_windows(&app, &overlays, &source);
    overlay_windows::emit_sprite_sequence_changed(&app, sequence.media_id);
    Ok(sequence)
}

/// Removes sprite-sheet playback metadata.
///
/// # Errors
/// Returns a command error when the item is unavailable or storage fails.
#[tauri::command]
pub(crate) async fn clear_sprite_sequence(
    app: AppHandle,
    ai_state: State<'_, AiState>,
    state: State<'_, AppState>,
    id: MediaId,
) -> Result<(), CommandError> {
    let _generated_media = ai_state.lock_generated_media().await;
    let (source, overlays) = with_library(state, move |library| {
        library.clear_sprite_sequence(id)?;
        let source = overlay_windows::OverlayMedia::load(library, id)?;
        let overlays = active_overlays_for(library, id)?;
        Ok((source, overlays))
    })
    .await?;
    sync_media_windows(&app, &overlays, &source);
    overlay_windows::emit_sprite_sequence_changed(&app, id);
    Ok(())
}

/// Lists generated variants for one immutable imported source.
///
/// # Errors
/// Returns a command error when variant metadata cannot be read.
#[tauri::command]
pub(crate) async fn list_media_variants(
    state: State<'_, AppState>,
    id: MediaId,
) -> Result<Vec<MediaVariant>, CommandError> {
    with_library(state, move |library| library.media_variants(id)).await
}

fn active_overlays_for(
    library: &lumadesk_core::Library,
    media_id: MediaId,
) -> Result<Vec<OverlayInstance>, lumadesk_core::CoreError> {
    Ok(library
        .list_overlays()?
        .into_iter()
        .filter(|overlay| overlay.media_id == media_id && overlay.is_active)
        .collect())
}

fn sync_media_windows(
    app: &AppHandle,
    overlays: &[OverlayInstance],
    source: &overlay_windows::OverlayMedia,
) {
    for overlay in overlays {
        if let Err(error) = overlay_windows::sync(app, overlay, source) {
            eprintln!(
                "failed to resize overlay {} after media changed: {error}",
                overlay.id
            );
        }
    }
}

fn restore_overlay_windows(
    app: &AppHandle,
    overlays: &[OverlayInstance],
    source: &overlay_windows::OverlayMedia,
) {
    for overlay in overlays {
        if let Err(error) = overlay_windows::sync(app, overlay, source) {
            eprintln!(
                "failed to restore overlay {} after delete failed: {error}",
                overlay.id
            );
        }
    }
}
