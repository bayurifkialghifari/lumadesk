use tauri::{
    AppHandle, Manager as _, Runtime, Window, WindowEvent,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::state::AppState;

const MAIN_WINDOW_LABEL: &str = "main";
const SHOW_MENU_ID: &str = "tray-show";
const QUIT_MENU_ID: &str = "tray-quit";

/// Shows, restores, and focuses the main management window when it exists.
pub(crate) fn show_main_window<R: Runtime>(application: &AppHandle<R>) -> tauri::Result<()> {
    let Some(window) = application.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };

    if window.is_minimized()? {
        window.unminimize()?;
    }
    window.show()?;
    window.set_focus()
}

/// Installs the cross-platform tray or menu-bar icon and its application menu.
pub(crate) fn install_tray<R: Runtime>(application: &AppHandle<R>) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(
        application,
        SHOW_MENU_ID,
        "Show LumaDesk",
        true,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(
        application,
        QUIT_MENU_ID,
        "Quit LumaDesk",
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(application)?;
    let menu = Menu::with_items(application, &[&show_item, &separator, &quit_item])?;
    let show_id = show_item.id().clone();
    let quit_id = quit_item.id().clone();

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("LumaDesk")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            if event.id() == &show_id {
                if let Err(error) = show_main_window(app) {
                    eprintln!("failed to show main window from tray: {error}");
                }
            } else if event.id() == &quit_id {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                if let Err(error) = show_main_window(tray.app_handle()) {
                    eprintln!("failed to show main window from tray icon: {error}");
                }
            }
            TrayIconEvent::Click { .. }
            | TrayIconEvent::DoubleClick { .. }
            | TrayIconEvent::Enter { .. }
            | TrayIconEvent::Move { .. }
            | TrayIconEvent::Leave { .. } => {}
            _ => eprintln!("ignored an unsupported tray icon event"),
        });

    if let Some(icon) = application.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(application)?;
    Ok(())
}

/// Hides only the main window when close-to-tray is enabled.
pub(crate) fn handle_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != MAIN_WINDOW_LABEL {
        return;
    }
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };

    let close_to_tray = match window.state::<AppState>().close_to_tray() {
        Ok(enabled) => enabled,
        Err(error) => {
            eprintln!("failed to read close-to-tray setting; using the enabled default: {error}");
            true
        }
    };
    if !close_to_tray {
        return;
    }

    api.prevent_close();
    if let Err(error) = window.hide() {
        eprintln!("failed to hide main window to tray: {error}");
    }
}
