# Window and application lifecycle

The custom title bar provides the desktop controls expected from a native application.

## Window controls

- **Minimize** sends the management window to the operating system's minimized state.
- **Maximize/Restore** toggles between the maximized and previous window size.
- **Close** hides the management window to the system tray or menu bar by default. Active overlay windows continue running.

The **Close to tray** setting controls this behavior. Turning it off makes **Close** end the normal main-window lifecycle instead. Overlay window close actions are independent and always close only the selected overlay.

## System tray and menu bar

The tray menu provides **Show LumaDesk** and **Quit LumaDesk**. **Show LumaDesk** restores a hidden or minimized management window and brings it to the foreground. **Quit LumaDesk** exits the process and closes its native windows. Left-clicking the tray icon also restores the window on platforms that emit tray click events; Linux desktop environments can always use the tray menu.

LumaDesk runs as a single application instance. Launching it again asks the existing process to restore and focus the main window, including when it is hidden in the tray.

## Layout restoration

The **Restore overlays** option activates previously active overlays when LumaDesk starts. It is enabled by default. System autostart is a separate setting and is disabled by default.
