use tauri::{
    AppHandle, Manager as _, Runtime, Window, WindowEvent,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use lumadesk_core::OverlayBatchPatch;

use crate::{commands::overlays::patch_all_overlays_impl, state::AppState};

const MAIN_WINDOW_LABEL: &str = "main";
const SHOW_MENU_ID: &str = "tray-show";
const SHOW_ALL_MENU_ID: &str = "tray-show-all";
const HIDE_ALL_MENU_ID: &str = "tray-hide-all";
const PLAY_ALL_MENU_ID: &str = "tray-play-all";
const PAUSE_ALL_MENU_ID: &str = "tray-pause-all";
const LOCK_ALL_MENU_ID: &str = "tray-lock-all";
const UNLOCK_ALL_MENU_ID: &str = "tray-unlock-all";
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
pub(crate) fn install_tray(application: &AppHandle) -> tauri::Result<()> {
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
    let show_all_item = MenuItem::with_id(
        application,
        SHOW_ALL_MENU_ID,
        "Show all overlays",
        true,
        None::<&str>,
    )?;
    let hide_all_item = MenuItem::with_id(
        application,
        HIDE_ALL_MENU_ID,
        "Hide all overlays",
        true,
        None::<&str>,
    )?;
    let play_all_item = MenuItem::with_id(
        application,
        PLAY_ALL_MENU_ID,
        "Play all overlays",
        true,
        None::<&str>,
    )?;
    let pause_all_item = MenuItem::with_id(
        application,
        PAUSE_ALL_MENU_ID,
        "Pause all overlays",
        true,
        None::<&str>,
    )?;
    let lock_all_item = MenuItem::with_id(
        application,
        LOCK_ALL_MENU_ID,
        "Lock all overlays",
        true,
        None::<&str>,
    )?;
    let unlock_all_item = MenuItem::with_id(
        application,
        UNLOCK_ALL_MENU_ID,
        "Unlock all overlays",
        true,
        None::<&str>,
    )?;
    let first_separator = PredefinedMenuItem::separator(application)?;
    let second_separator = PredefinedMenuItem::separator(application)?;
    let menu = Menu::with_items(
        application,
        &[
            &show_item,
            &first_separator,
            &show_all_item,
            &hide_all_item,
            &play_all_item,
            &pause_all_item,
            &lock_all_item,
            &unlock_all_item,
            &second_separator,
            &quit_item,
        ],
    )?;
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
            } else {
                let patch = match event.id().as_ref() {
                    SHOW_ALL_MENU_ID => OverlayBatchPatch {
                        is_active: Some(true),
                        ..OverlayBatchPatch::default()
                    },
                    HIDE_ALL_MENU_ID => OverlayBatchPatch {
                        is_active: Some(false),
                        ..OverlayBatchPatch::default()
                    },
                    PLAY_ALL_MENU_ID => OverlayBatchPatch {
                        is_playing: Some(true),
                        ..OverlayBatchPatch::default()
                    },
                    PAUSE_ALL_MENU_ID => OverlayBatchPatch {
                        is_playing: Some(false),
                        ..OverlayBatchPatch::default()
                    },
                    LOCK_ALL_MENU_ID => OverlayBatchPatch {
                        click_through: Some(true),
                        ..OverlayBatchPatch::default()
                    },
                    UNLOCK_ALL_MENU_ID => OverlayBatchPatch {
                        click_through: Some(false),
                        ..OverlayBatchPatch::default()
                    },
                    _ => return,
                };
                let application = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = patch_all_overlays_impl(&application, patch).await {
                        eprintln!("failed to update overlays from tray: {error:?}");
                    }
                });
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
