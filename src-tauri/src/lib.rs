//! Native application adapter for `LumaDesk`.

use tauri::Manager as _;

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
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            eprintln!("LumaDesk failed to start: {error}");
            std::process::exit(1);
        });
}
