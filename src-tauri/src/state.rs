use std::sync::{Arc, Mutex};

use lumadesk_core::{CoreError, Library};
use tauri::State;
use thiserror::Error;

use crate::command_error::CommandError;

#[derive(Debug)]
pub(crate) struct AppState {
    library: Arc<Mutex<Library>>,
}

#[derive(Debug, Error)]
pub(crate) enum SettingsAccessError {
    #[error("application state is temporarily unavailable")]
    Unavailable,
    #[error("application settings could not be read: {0}")]
    Core(#[from] CoreError),
}

impl AppState {
    pub(crate) fn new(library: Library) -> Self {
        Self {
            library: Arc::new(Mutex::new(library)),
        }
    }

    /// Reads the close-to-tray preference for synchronous window events.
    pub(crate) fn close_to_tray(&self) -> Result<bool, SettingsAccessError> {
        let library = self
            .library
            .lock()
            .map_err(|_| SettingsAccessError::Unavailable)?;
        Ok(library.get_settings()?.close_to_tray)
    }
}

pub(crate) async fn with_library<T, F>(
    state: State<'_, AppState>,
    operation: F,
) -> Result<T, CommandError>
where
    T: Send + 'static,
    F: FnOnce(&mut Library) -> Result<T, CoreError> + Send + 'static,
{
    let library = Arc::clone(&state.library);
    tauri::async_runtime::spawn_blocking(move || {
        let mut library = library.lock().map_err(|_| CommandError::unavailable())?;
        operation(&mut library).map_err(CommandError::from)
    })
    .await
    .map_err(|_| CommandError::unavailable())?
}
