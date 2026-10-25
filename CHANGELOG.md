# Changelog

All notable changes to LumaDesk are documented here. The project follows Semantic Versioning and keeps unreleased work at the top.

## [Unreleased]

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

[Unreleased]: https://github.com/bayurifkialghifari/lumadesk/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/bayurifkialghifari/lumadesk/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/bayurifkialghifari/lumadesk/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/bayurifkialghifari/lumadesk/releases/tag/v0.1.0
