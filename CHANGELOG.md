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
- Native transparent overlay windows with centered initial placement, always-on-top display, click-through input, persisted restore, and cleanup when media is deleted.
- Canvas overlay playback with original frame timing, per-instance speed, pause, opacity, horizontal flip, looping, and a bounded two-batch memory buffer.
- **Tampilkan di desktop** action in the media inspector with native success and error feedback.
