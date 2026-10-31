use std::fs;

use lumadesk_core::AppSettings;
use tauri::{AppHandle, Manager as _, State};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_opener::OpenerExt as _;

use crate::{
    ai::AiState,
    command_error::CommandError,
    state::{AppState, with_library},
};

/// Reads typed application settings.
///
/// # Errors
/// Returns a command error when local settings cannot be read.
#[tauri::command]
pub(crate) async fn get_settings(
    application: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppSettings, CommandError> {
    let mut settings = with_library(state.clone(), |library| library.get_settings()).await?;
    let launch_at_login = launch_at_login_enabled(&application)?;
    if settings.launch_at_login == launch_at_login {
        return Ok(settings);
    }

    settings.launch_at_login = launch_at_login;
    with_library(state, move |library| library.update_settings(&settings)).await
}

/// Validates and persists application settings.
///
/// # Errors
/// Returns a command error for invalid values or unavailable storage.
#[tauri::command]
pub(crate) async fn update_settings(
    application: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, CommandError> {
    let previous_launch_state = launch_at_login_enabled(&application)?;
    let launch_state_changed = settings.launch_at_login != previous_launch_state;
    if launch_state_changed {
        set_launch_at_login(&application, settings.launch_at_login)?;
    }

    let saved = with_library(state, move |library| library.update_settings(&settings)).await;
    if saved.is_err()
        && launch_state_changed
        && let Err(error) = set_launch_at_login(&application, previous_launch_state)
    {
        eprintln!("failed to restore launch-at-login state: {error:?}");
    }
    saved
}

/// Opens the application data directory in the platform file manager.
///
/// # Errors
/// Returns a command error when the directory cannot be prepared or opened.
#[tauri::command]
pub(crate) fn open_data_folder(application: AppHandle) -> Result<(), CommandError> {
    let data_root = application
        .path()
        .app_data_dir()
        .map_err(|error| CommandError::native("The data folder could not be located", error))?;
    fs::create_dir_all(&data_root)
        .map_err(|error| CommandError::native("The data folder could not be prepared", error))?;
    application
        .opener()
        .open_path(data_root.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|error| CommandError::native("The data folder could not be opened", error))
}

/// Removes generated previews while preserving imported media and the local database.
///
/// # Errors
/// Returns a command error when generated files cannot be removed.
#[tauri::command]
pub(crate) async fn clear_generated_cache(
    ai_state: State<'_, AiState>,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let _generated_media = ai_state.lock_generated_media().await;
    with_library(state, |library| library.clear_generated_cache()).await
}

fn launch_at_login_enabled(application: &AppHandle) -> Result<bool, CommandError> {
    application
        .autolaunch()
        .is_enabled()
        .map_err(|error| CommandError::native("The launch-at-login state could not be read", error))
}

fn set_launch_at_login(application: &AppHandle, enabled: bool) -> Result<(), CommandError> {
    let result = if enabled {
        application.autolaunch().enable()
    } else {
        application.autolaunch().disable()
    };
    result.map_err(|error| {
        CommandError::native("The launch-at-login state could not be updated", error)
    })
}
