use lumadesk_core::AppSettings;
use tauri::State;

use crate::{
    command_error::CommandError,
    state::{AppState, with_library},
};

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
