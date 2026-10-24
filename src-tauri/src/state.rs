use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use lumadesk_core::{CoreError, Library};
use tauri::State;

use crate::command_error::CommandError;

#[derive(Debug)]
pub(crate) struct AppState {
    library: Arc<Mutex<Library>>,
    overlay_edit_mode: AtomicBool,
}

impl AppState {
    pub(crate) fn new(library: Library) -> Self {
        Self {
            library: Arc::new(Mutex::new(library)),
            overlay_edit_mode: AtomicBool::new(false),
        }
    }

    pub(crate) fn overlay_edit_mode(&self) -> bool {
        self.overlay_edit_mode.load(Ordering::Acquire)
    }

    pub(crate) fn set_overlay_edit_mode(&self, enabled: bool) {
        self.overlay_edit_mode.store(enabled, Ordering::Release);
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
