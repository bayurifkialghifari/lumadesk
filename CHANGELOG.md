# Changelog

All notable changes to LumaDesk are documented here. The project follows Semantic Versioning and keeps unreleased work at the top.

## [Unreleased]

### Added

- Added startup update checks and a title-bar action that downloads, verifies, installs, and relaunches into a newer release.
- Added determinate download progress, quiet offline behavior, retryable installation errors, and responsive update states.
- Added signed updater archives and a generated `latest.json` feed for Windows x64 and ARM64, macOS Apple Silicon and Intel, and Linux x86-64 and ARM64.
- Added an application-update guide covering package verification, installation behavior, release targets, and recovery.

### Changed

- Tagged releases now publish direct installers, updater payloads, detached signatures, the update manifest, and SHA-256 checksums together.
- Raised the minimum Rust toolchain to 1.90 for the native updater runtime.

## [0.5.1] - 2026-10-31

### Added

- Added a visual product showcase to the README with the Collection, Creator Studio, Overlay Console, animation details, desktop overlay, and Settings workspaces.

### Fixed

- Background removal can read GIF and image pixels from Tauri asset URLs again on Windows, macOS, and Linux.
- Frame loading now keeps its image fallback origin-clean when bitmap decoding is unavailable.

## [0.5.0] - 2026-10-31

### Added

- Added named desktop scenes that capture the complete overlay layout and support apply, update, rename, duplicate, and delete actions.
- Added monitor-aware placement with a per-overlay display selector, relative-position mapping, full-bounds clamping, and primary-display fallback.
- Added click-through overlays with recovery controls in the overlay toolbar, desktop manager, and system tray.
- Added global show, hide, play, pause, lock, unlock, and confirmed close actions in the desktop manager.
- Added matching show, hide, play, pause, lock, and unlock actions to the tray or menu-bar menu.

### Changed

- **Active on Desktop** now lists hidden overlays and shows total, visible, playing, and locked counts.
- Scene snapshots now preserve visibility, playback, background removal, click-through, z-order, and monitor placement.
- Scene application replaces the current desktop layout and restores the previous database and native windows if synchronization fails.

### Fixed

- Saved scenes now retain hidden overlay state across updates and application.
- The HTML language and application description now use English.

## [0.4.0] - 2026-10-31

### Added

- Added Creator Studio with non-destructive crop, quarter-turn rotation, horizontal flip, brightness, contrast, saturation, background modes, live preview, reset, and explicit save.
- Added sprite-sheet setup with configurable grids, frame counts, timing, and loop, once, or ping-pong playback across previews and desktop overlays.
- Added functional Settings for theme, Collection view, startup behavior, overlay defaults, and storage actions.
- Added optional local U²-NetP and U²-Net background models with verified downloads, removable model storage, bounded mask generation, and local ONNX inference.
- Added SQLite edit recipes, sprite sequences, generated media variants, and transactional Creator saves.

### Changed

- Collection previews and desktop overlays now apply saved edit recipes and sprite playback.
- Generated-media operations now coordinate cache cleanup, model removal, media deletion, Creator updates, and mask generation to prevent conflicting file changes.
- Cache cleanup now removes generated previews, decoded frames, and segmentation masks while preserving installed models and source media.
- Creator crop bounds now use one decoded sprite frame, and changing the sprite grid clamps the crop to the new cell dimensions.

### Fixed

- Invalid sprite metadata can no longer leave a recipe partially saved.
- Storage actions now always leave their busy state after success or failure.

## [0.3.0] - 2026-10-25

### Added

- Added a **Discover GIFs** Collection action that opens Tenor in the default browser and carries over the current search text.
- Added a system tray or menu-bar lifecycle with **Show LumaDesk** and **Quit LumaDesk** actions. Closing the main window now hides it to the tray by default while overlays continue running.

### Changed

- Overlay edit controls and boundaries now appear when the animation is pressed and hide after three seconds without interaction.

## [0.2.1] - 2026-10-25

### Added

- Added a background-removal toggle directly to every desktop overlay while keeping the matching control in **Active on Desktop**.

### Fixed

- Frame images now use anonymous CORS before loading, allowing background removal to read pixels from Tauri asset URLs without tainting the canvas.
- The title bar now reads the application version from package metadata instead of displaying a stale hardcoded value.

### Removed

- Removed the nonstandard Hide title-bar action; window controls now use Minimize, Maximize/Restore, and Close.

## [0.2.0] - 2026-10-25

### Added

- Per-overlay automatic background removal for still images and animated frames, with edge-connected color detection, feathered transparency, bounded frame caching, and unchanged source media.
- Custom title-bar controls for Hide, Minimize, Maximize/Restore, and Close.
- English application copy, user-facing errors, and public documentation.

### Changed

- Overlays now accept pointer input at all times and can be moved directly without a separate edit mode.
- Overlay controls now remain available in a permanent compact toolbar and boundary.
- Removed the redundant privacy message from the sidebar.

## [0.1.0] - 2026-10-25

### Added

- Initial product, design, architecture, and user-documentation contracts.
- Original LumaDesk visual identity.
- Feature documentation for library, import, overlay, lifecycle, settings, and release behavior.
- Local SQLite catalog with transactional schema migration, search, tags, favorites, typed settings, and persisted overlay records.
- Managed media import for GIF, APNG, WebP, PNG, and JPEG with SHA-256 duplicate detection and bounded frame inspection.
- Coordinated media deletion that removes related overlays and managed data while preserving the original source file.
- Typed Tauri command bridge and validated TypeScript client for catalog, overlay, media-path, and settings operations.
- Responsive collection workspace with search, filters, grid and list views, pagination, native file import, drag and drop, item inspection, rename, tag, favorite, delete, loading, error, and import-result states.
- Persistent frame decoding cache with atomic generation, bounded 64-frame batches, original frame timing, and reusable asset URLs for native overlays.
- Native transparent overlay windows with centered initial placement, always-on-top display, persisted restore, and cleanup when media is deleted.
- Canvas overlay playback with original frame timing, per-instance speed, pause, opacity, horizontal flip, looping, and a bounded two-batch memory buffer.
- **Show on desktop** action in the media inspector with native success and error feedback.
- **Active on Desktop** workspace with live instance counts, responsive cards, browser-preview states, per-instance controls, and pause-all or confirmed close-all actions.
- GitHub Actions checks and tagged release builds for Windows x86-64 and ARM64, macOS Apple Silicon and Intel, and Linux x86-64 and ARM64.

[Unreleased]: https://github.com/bayurifkialghifari/lumadesk/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/bayurifkialghifari/lumadesk/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/bayurifkialghifari/lumadesk/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/bayurifkialghifari/lumadesk/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/bayurifkialghifari/lumadesk/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/bayurifkialghifari/lumadesk/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/bayurifkialghifari/lumadesk/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/bayurifkialghifari/lumadesk/releases/tag/v0.1.0
