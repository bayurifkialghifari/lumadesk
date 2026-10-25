# LumaDesk Product Requirements

**Status:** v0.2 implementation contract
**Updated:** 25 October 2026

## Product statement

LumaDesk is a local-first desktop animation library. It lets people collect transparent or image-based animations, preview them, and place any number of them as lightweight always-on-top overlays on the desktop. The same library and layout return after the app is reopened.

## Users and jobs

- Desktop customizers want ambient characters, effects, and pixel loops without configuring a streaming tool.
- Artists want to test GIF, APNG, WebP, and still-image assets against a real desktop.
- Collectors want one searchable library with names, tags, favorites, and safe local ownership.

## v0.2 outcomes

1. A user can import GIF, APNG, animated/static WebP, PNG, and JPG files through a file picker or drag and drop.
2. Imported media appears in a searchable library and can be previewed, renamed, tagged, favorited, or deleted.
3. A library item can create one or more transparent desktop overlays with independent position, size, opacity, speed, flip, pause, and loop controls.
4. Overlay layouts survive application restarts with persisted logical positions.
5. All media, settings, and catalog data stay on the user's device.
6. Installers are built for Linux, macOS, and Windows from version tags.
7. Each overlay can remove a solid or near-solid edge-connected background without changing the imported file.

## Functional requirements

### Library

- Show starter and imported items in grid and compact-list views.
- Filter by text, favorite state, file format, and tags.
- Open an external GIF search from the Collection toolbar, carrying the current text query into Tenor when one is present. Downloaded files return through the normal local import flow.
- Detect duplicate imports by SHA-256 and focus the existing item instead of copying it again.
- Store imported files in an application-managed media directory. Renaming changes the display name only.
- Require confirmation before deletion. Deleting an item stops its active overlays and removes its managed media and cache files; the original source file is never touched.

### Preview and overlays

- Preserve transparency, frame order, frame duration, looping, and GIF/APNG disposal behavior.
- Render still images as valid one-frame animations.
- Create one undecorated transparent native window for each active overlay.
- Keep every overlay interactive so it can be dragged directly from the animation or toolbar.
- Keep a compact overlay toolbar and boundary visible for play, pause, flip, duplicate, close, and drag actions.
- Expose playback, 25–400% speed, 10–100% opacity, horizontal flip, and bounded size controls.
- Expose a persisted **Remove background** checkbox per overlay for still images, GIF, APNG, and animated WebP frames.
- Estimate the background from frame edges, remove only matching regions connected to those edges, feather the result, and preserve existing transparency.
- Keep imported and managed source media unchanged when an effect is enabled or disabled.
- Restore active instances when LumaDesk starts. Monitor recovery remains deferred until saved positions can be remapped safely.

### Desktop lifecycle

- Run as a single application instance. Reopening focuses the library window.
- Provide Hide, Minimize, Maximize/Restore, and Close controls in the custom title bar.
- Autostart is opt-in and disabled by default.
- Linux targets X11 directly and XWayland automatically when launched in a Wayland session.

### Settings

- Persist theme, library view, default overlay size, default speed, default opacity, restore-layout preference, autostart preference, and cache limit.
- Provide cache cleanup and a link that opens the application data directory.

## Quality requirements

- Library actions should feel immediate for collections of at least 1,000 items.
- Decoding must be bounded by a frame cache; the app may not hold every frame of every active item indefinitely.
- Background-removed frames must follow the same bounded playback batches and may be cached only while their frame batch remains active.
- Database migrations are transactional and preserve user data.
- Corrupt and unsupported files return a clear user-facing error without creating partial catalog entries.
- Keyboard navigation, visible focus, reduced motion, labels, contrast, and minimum 40 px interaction targets are required.
- The application interface and all user-facing errors use English.

## Distribution and privacy

- Linux x86-64 and ARM64: AppImage, Debian, and RPM packages.
- Windows x86-64 and ARM64: separate NSIS installers.
- macOS Apple Silicon and Intel: separate DMG packages.
- The app does not require an account, analytics, telemetry, cloud storage, or network access for core functionality.
- Signing and notarization activate when repository secrets are provided; unsigned development artifacts remain identifiable as such.

## Deferred

- Timeline, sprite, or animation editing.
- AI subject segmentation for complex photographic backgrounds.
- Video and audio playback.
- Autonomous pets, desktop object detection, and interactive character AI.
- Online catalog, account sync, cloud backup, and marketplace features.
- Native Wayland layer-shell integrations.

## Acceptance criteria

- All v0.2 outcomes pass automated tests where the behavior is deterministic.
- Overlay behavior is manually verified on Windows, macOS, Linux X11, and Linux Wayland through XWayland.
- A fresh machine can install a tagged release, import media, place an overlay, restart, and receive the same layout.
- README download links, release assets, checksums, and changelog agree with the released version.
