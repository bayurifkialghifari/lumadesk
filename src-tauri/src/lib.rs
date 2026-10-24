//! Native application adapter for `LumaDesk`.

mod command_error;
mod commands;
mod overlay_windows;
mod state;

use lumadesk_core::Library;
use tauri::Manager as _;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(
            |application, _arguments, _cwd| {
                if let Some(window) = application.get_webview_window("main") {
                    if let Err(error) = window.show() {
                        eprintln!("failed to show main window: {error}");
                    }
                    if let Err(error) = window.set_focus() {
                        eprintln!("failed to focus main window: {error}");
                    }
                }
            },
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .setup(|application| {
            let data_root = application.path().app_data_dir()?;
            let library = Library::open(&data_root)?;
            let restore = library.get_settings()?.restore_overlays;
            let overlays = if restore {
                library
                    .list_overlays()?
                    .into_iter()
                    .filter(|overlay| overlay.is_active)
                    .map(|overlay| {
                        let media = library.media_item(overlay.media_id)?;
                        Ok((overlay, media))
                    })
                    .collect::<Result<Vec<_>, lumadesk_core::CoreError>>()?
            } else {
                Vec::new()
            };
            application.manage(AppState::new(library));
            for (overlay, media) in overlays {
                if let Err(error) = overlay_windows::sync(application.handle(), &overlay, &media) {
                    eprintln!("failed to restore overlay {}: {error}", overlay.id);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_media,
            commands::import_media,
            commands::rename_media,
            commands::set_favorite,
            commands::replace_tags,
            commands::delete_media,
            commands::resolve_media_path,
            commands::get_frame_batch,
            commands::list_overlays,
            commands::list_desktop_overlays,
            commands::create_overlay,
            commands::show_media_overlay,
            commands::update_overlay,
            commands::move_overlay,
            commands::duplicate_overlay,
            commands::remove_overlay,
            commands::get_overlay_edit_mode,
            commands::set_overlay_edit_mode,
            commands::get_settings,
            commands::update_settings,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            eprintln!("LumaDesk failed to start: {error}");
            std::process::exit(1);
        });
}
