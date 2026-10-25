# Changelog

All notable changes to LumaDesk are documented here. The project follows Semantic Versioning and keeps unreleased work at the top.

## [Unreleased]

### Added

- Initial product, design, architecture, and user-documentation contracts.
- Original LumaDesk visual identity.
- Feature documentation organized by library, import, overlay, lifecycle, settings, and release behavior.
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
- Always-interactive overlays with direct native drag placement, persisted logical positions, a permanent boundary, and compact play, flip, duplicate, and close controls.
- Custom title-bar controls for Hide, Minimize, Maximize/Restore, and Close.
- English interface copy and user-facing error messages throughout the application.

### Changed

- Removed the global interaction-mode toggle so overlays can be moved directly at any time.
- Removed the redundant privacy message from the sidebar.
- GitHub Actions checks for frontend and Rust code plus manual or tagged builds for Windows x86-64 and ARM64, macOS Apple Silicon and Intel, and Linux x86-64 and ARM64. Tagged builds validate synchronized versions, generate SHA-256 checksums, and publish GitHub Releases automatically.
