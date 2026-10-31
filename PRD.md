# LumaDesk Product Requirements

**Status:** v0.6.0 implementation contract
**Updated:** 31 October 2026

## Product statement

LumaDesk is a desktop animation library and editor. It organizes local media, applies reusable non-destructive edits, turns sprite sheets into animations, and places any number of items on the desktop as lightweight always-on-top overlays.

## Users and jobs

- Desktop customizers want ambient characters, effects, and pixel loops without configuring a streaming tool.
- Artists want to prepare GIF, APNG, WebP, still-image, and sprite-sheet assets while preserving their source files.
- Collectors want a searchable library with names, tags, favorites, duplicate detection, and predictable local ownership.

## v0.6.0 outcomes

1. Users can import GIF, APNG, animated or static WebP, PNG, and JPEG files through a native picker or drag and drop.
2. Imported media appears in a searchable library and can be previewed, renamed, tagged, favorited, or removed.
3. Creator Studio can crop, rotate, flip, adjust color, choose a background-removal mode, and configure sprite playback without rewriting source media.
4. Still images can become sprite animations with configurable grids, frame counts, timing, and loop, once, or ping-pong playback.
5. Optional U²-Net models can be downloaded, verified, used locally for subject masks, and removed from Settings.
6. Library items can create independent desktop overlays with persisted position, size, opacity, speed, playback, flip, and background controls.
7. Settings control appearance, startup behavior, overlay defaults, storage, cache cleanup, and local AI models.
8. Version tags build direct installers for supported Windows, macOS, and Linux architectures.
9. Users can save, apply, update, rename, duplicate, and delete complete desktop scenes.
10. Overlay placement can move between connected monitors and falls back safely when a saved display is unavailable.
11. The manager and tray can show, hide, play, pause, lock, or unlock every persisted overlay in one action.
12. Click-through overlays pass pointer input to windows below and remain recoverable from the manager and tray.
13. Installed builds check GitHub Releases for signed updates and offer one-click installation from the title bar.

## Functional requirements

### Library and discovery

- Show starter and imported items in grid and compact-list views.
- Filter by text, favorite state, asset kind, file format, and tags.
- Open an external GIF search from the Collection toolbar and pass the current search text to Tenor when present.
- Detect duplicate imports by SHA-256 and focus the existing item instead of creating another copy.
- Store imported files in an application-managed media directory. Renaming changes the catalog name only.
- Confirm removal, stop dependent overlays, and delete the managed copy and regenerable data without touching the original source file.

### Creator Studio

- Provide an asset rail, a live preview, explicit reset controls, and an explicit **Save changes** action.
- Persist crop, quarter-turn rotation, horizontal flip, brightness, contrast, saturation, and background-removal mode as an edit recipe.
- Support original, edge-connected, AI fast, and AI quality background modes.
- Validate edits and sprite metadata before committing both in one SQLite transaction. A failed save must leave the previous recipe and sprite configuration intact.
- Invalidate decoded sprite frames and generated segmentation masks when sprite metadata changes.
- Keep every edit non-destructive; managed and original source files remain unchanged.

### Sprite sheets

- Allow still images and existing sprite sheets to use sprite playback.
- Read frames left to right and top to bottom from a configurable grid.
- Validate columns, rows, frame count, per-frame duration, and source bounds.
- Support loop, once, and ping-pong playback in previews and desktop overlays.
- Apply crop coordinates within one decoded sprite cell rather than across the complete sheet.
- Disable sprite setup for animated source formats that already contain a frame sequence.

### Background removal

- Keep the immediate edge-connected effect available without a model download.
- Offer optional Fast and Quality models through Settings.
- Verify every model download against its expected byte length and SHA-256 digest before inference.
- Run ONNX inference on the device and store generated masks as regenerable variants.
- Reuse masks for matching source and sprite metadata, and discard stale masks after relevant edits.
- Serialize generated-media operations so model removal, cache cleanup, editing, and mask generation cannot corrupt shared files or records.

### Preview and overlays

- Preserve transparency, frame order, frame duration, looping, and GIF/APNG disposal behavior.
- Render still images as valid one-frame animations.
- Apply the saved edit recipe and sprite sequence to Collection previews and desktop overlays.
- Create one undecorated transparent native window for each active overlay.
- Let users drag an overlay directly from the animation surface. Pressing the overlay reveals edit controls, and idle controls hide after three seconds.
- Expose playback, 25–400% speed, 10–100% opacity, horizontal flip, background removal, duplication, and bounded size controls.
- Expose visible, hidden, playing, paused, interactive, and click-through states in the desktop manager.
- Let the user choose a connected display for each overlay while preserving its relative position and clamping the complete overlay inside the target display.
- Support batch show, hide, play, pause, lock, unlock, and confirmed close actions across all persisted overlays.
- Restore active instances at startup when layout restoration is enabled.

### Desktop scenes

- Capture every persisted overlay in a named scene, including media, monitor, normalized position, size, opacity, speed, flip, background removal, playback, click-through, z-order, and visibility.
- Apply a scene as a complete desktop replacement instead of layering it over the current layout.
- Resolve saved monitor names against connected displays and fall back to the primary display when a saved monitor is unavailable.
- Keep scene names unique without case sensitivity and support update, rename, duplicate, and confirmed delete actions.
- Apply scene database and native-window changes as one coordinated operation and restore the previous layout if native synchronization fails.

### Desktop lifecycle and settings

- Run as one application instance and focus the existing management window when reopened.
- Provide Minimize, Maximize/Restore, and Close controls in the custom title bar.
- Keep the application in the system tray or menu bar after the main window closes when **Close to tray** is enabled.
- Provide tray actions to show, hide, play, pause, lock, and unlock all overlays while the management window is closed.
- Make autostart opt-in and disabled by default.
- Persist theme, Collection view, overlay defaults, restore behavior, close-to-tray behavior, and autostart.
- Let users open the application data directory, clear regenerable caches, and install or remove local AI models.
- Target X11 directly and use XWayland when launched in a Wayland session.
- Check for an application update at startup without blocking or interrupting the workspace.
- Replace the title-bar version label with an update action when a newer signed release is available.
- Show determinate download progress when the package size is known, install only after a user action, verify the package signature, and relaunch after installation.
- Treat an unavailable update service as a quiet offline state and report installation failures through the standard application toast.

## Quality requirements

- Library actions should remain responsive for collections of at least 1,000 items.
- Frame and mask processing must use bounded batches and caches rather than retaining complete animated collections in memory.
- Database migrations and Creator saves are transactional.
- Corrupt, unsupported, or invalid media returns a clear English error without leaving partial catalog data.
- File paths received from the frontend may never grant arbitrary filesystem access.
- Keyboard navigation, visible focus, reduced motion, labels, contrast, and minimum 40 px primary interaction targets are required.
- The interface, errors, documentation, and release notes use English.

## Distribution and privacy

- Windows x86-64 and ARM64: separate NSIS `.exe` installers.
- macOS Apple Silicon and Intel: separate `.dmg` packages.
- Linux x86-64 and ARM64: `.AppImage`, `.deb`, and `.rpm` packages.
- Every tagged release includes signed updater payloads and a static update manifest for all six supported platform and architecture combinations.
- Core library, editing, playback, and edge removal do not require an account or network connection.
- Network access is limited to update checks, user-initiated external discovery, optional model downloads, and downloading an update after user confirmation.
- Signing and notarization activate when repository secrets are configured.

## Deferred

- Timeline and per-frame drawing tools.
- Video and audio playback.
- Embedded online marketplace, accounts, sync, and cloud backup.
- Autonomous characters, desktop object detection, and conversational AI.
- Native Wayland layer-shell integration.

## Acceptance criteria

- Deterministic v0.6.0 behavior passes Rust and frontend tests.
- Invalid Creator input cannot partially save an edit recipe.
- A downloaded model must pass size and SHA-256 checks before it is reported as installed.
- A tagged release publishes direct installer files, signed updater payloads, `latest.json`, and `SHA256SUMS.txt` for every supported target.
- The tag, Cargo workspace, `package.json`, Tauri configuration, title-bar version, changelog, and release title agree.
- Applying a scene preserves hidden state, maps missing displays to the primary monitor, and never leaves the database and native windows on different layouts.
- A click-through overlay can always be unlocked from the management window or tray.
- A newer valid release appears as a title-bar update action; its package is verified before installation and the app relaunches when installation completes.
