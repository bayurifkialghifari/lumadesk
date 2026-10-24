use tauri::{AppHandle, State};

use crate::{command_error::CommandError, overlay_windows, state::AppState};

/// Returns whether desktop overlays currently accept edit input.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub(crate) fn get_overlay_edit_mode(state: State<'_, AppState>) -> bool {
    state.overlay_edit_mode()
}

/// Enables or disables pointer interaction for every desktop overlay.
///
/// # Errors
/// Returns a command error when native window interaction cannot be updated.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub(crate) fn set_overlay_edit_mode(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<bool, CommandError> {
    let previous = state.overlay_edit_mode();
    if previous == enabled {
        return Ok(enabled);
    }
    state.set_overlay_edit_mode(enabled);
    if let Err(error) = overlay_windows::apply_edit_mode(&app, enabled) {
        state.set_overlay_edit_mode(previous);
        if let Err(rollback_error) = overlay_windows::apply_edit_mode(&app, previous) {
            eprintln!("failed to restore overlay edit mode: {rollback_error}");
        }
        return Err(CommandError::overlay(&error));
    }
    Ok(enabled)
}
