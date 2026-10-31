//! Native application adapter for `LumaDesk`.

mod ai;
mod app_lifecycle;
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
                if let Err(error) = app_lifecycle::show_main_window(application) {
                    eprintln!("failed to restore main window: {error}");
                }
            },
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .setup(|application| {
            let data_root = application.path().app_data_dir()?;
            let library = Library::open(&data_root)?;
            let ai_state = ai::AiState::open(&data_root)?;
            let restore = library.get_settings()?.restore_overlays;
            let overlays = if restore {
                library
                    .list_overlays()?
                    .into_iter()
                    .filter(|overlay| overlay.is_active)
                    .map(|overlay| {
                        let source =
                            overlay_windows::OverlayMedia::load(&library, overlay.media_id)?;
                        Ok((overlay, source))
                    })
                    .collect::<Result<Vec<_>, lumadesk_core::CoreError>>()?
            } else {
                Vec::new()
            };
            application.manage(AppState::new(library));
            application.manage(ai_state);
            app_lifecycle::install_tray(application.handle())?;
            for (overlay, source) in overlays {
                if let Err(error) = overlay_windows::sync(application.handle(), &overlay, &source) {
                    eprintln!("failed to restore overlay {}: {error}", overlay.id);
                }
            }
            Ok(())
        })
        .on_window_event(app_lifecycle::handle_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::catalog::list_media,
            commands::catalog::import_media,
            commands::catalog::rename_media,
            commands::catalog::set_favorite,
            commands::catalog::replace_tags,
            commands::catalog::delete_media,
            commands::catalog::resolve_media_path,
            commands::catalog::get_frame_batch,
            commands::catalog::get_edit_recipe,
            commands::catalog::update_edit_recipe,
            commands::catalog::save_creator_changes,
            commands::catalog::get_sprite_sequence,
            commands::catalog::set_sprite_sequence,
            commands::catalog::clear_sprite_sequence,
            commands::catalog::list_media_variants,
            commands::overlays::list_overlays,
            commands::overlays::list_desktop_overlays,
            commands::overlays::create_overlay,
            commands::overlays::show_media_overlay,
            commands::overlays::update_overlay,
            commands::overlays::move_overlay,
            commands::overlays::duplicate_overlay,
            commands::overlays::remove_overlay,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::open_data_folder,
            commands::settings::clear_generated_cache,
            ai::list_ai_models,
            ai::download_ai_model,
            ai::remove_ai_model,
            ai::get_ai_mask_batch,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            eprintln!("LumaDesk failed to start: {error}");
            std::process::exit(1);
        });
}
