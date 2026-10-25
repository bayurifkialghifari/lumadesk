# Window and application lifecycle

The custom title bar provides the desktop controls expected from a native application.

## Window controls

- **Hide** removes the management window from view while leaving its native window available to the running application.
- **Minimize** sends the management window to the operating system's minimized state.
- **Maximize/Restore** toggles between the maximized and previous window size.
- **Close** closes the management window. Active overlay windows follow the native application lifecycle.

LumaDesk runs as a single application instance. Launching it again asks the existing process to show the main window.

## Layout restoration

The **Restore layout** option activates previously active overlays when LumaDesk starts. It is enabled by default. System autostart is a separate setting and is disabled by default.
