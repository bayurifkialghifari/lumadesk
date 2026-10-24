# LumaDesk Architecture

## System shape

LumaDesk is a Tauri 2 desktop application. Svelte 5 renders the management window and small overlay control surfaces. Rust owns trusted boundaries: files, SQLite, media decoding, cache management, application lifecycle, and native windows.

```mermaid
flowchart LR
  UI[Svelte management UI] -->|typed Tauri commands| APP[Rust application layer]
  APP --> DB[(SQLite catalog)]
  APP --> MEDIA[Managed media store]
  APP --> CACHE[Thumbnail and frame cache]
  APP --> WINDOWS[Native overlay windows]
  WINDOWS --> RENDER[Canvas frame renderer]
  APP --> TRAY[Tray and lifecycle]
```

## Repository boundaries

- `src/`: Svelte application, typed command client, stores, pages, components, and CSS tokens.
- `src-tauri/src/`: Tauri adapter, commands, tray, single-instance handling, and native window coordination.
- `crates/lumadesk-core/`: platform-independent domain, SQLite repositories, import pipeline, hashing, and media metadata.
- `assets/starter/`: redistributable starter animation sources and license records.
- `docs/`: user-facing feature and contribution guides.

The core crate has no Tauri dependency. Its database and import behavior can run in tests without a graphical session.

## Data model

### `media_items`

| Field | Meaning |
| --- | --- |
| `id` | UUID primary key |
| `content_hash` | unique SHA-256 of imported bytes |
| `display_name` | user-editable catalog name |
| `managed_path` | relative path inside the media root |
| `media_kind` | gif, apng, webp, png, or jpeg |
| `width`, `height` | decoded pixel dimensions |
| `frame_count` | number of decoded frames |
| `duration_ms` | total loop duration |
| `is_favorite` | favorite filter state |
| `created_at`, `updated_at` | UTC timestamps |

Tags use `tags` and `media_tags` tables. `managed_path` is never accepted from the frontend; the backend resolves it from the item ID.

### `overlay_instances`

Each record references a media item and stores logical x/y position, monitor identity, width, opacity, speed, horizontal flip, playback state, z-order, and active state. Logical coordinates avoid DPI-dependent persistence.

### `settings`

A key/value table stores typed application preferences. The application layer parses every value into a typed settings structure before exposing it.

## Command surface

The frontend calls serializable commands with typed inputs and outputs:

- `list_media(query) -> MediaPage`
- `import_media(paths) -> ImportReport`
- `rename_media(input) -> MediaItem`
- `set_favorite(input) -> MediaItem`
- `replace_tags(input) -> MediaItem`
- `delete_media(input) -> DeleteReport`
- `list_overlays() -> OverlayInstance[]`
- `create_overlay(input) -> OverlayInstance`
- `update_overlay(input) -> OverlayInstance`
- `remove_overlay(id) -> void`
- `set_edit_mode(enabled) -> void`
- `get_settings() -> AppSettings`
- `update_settings(patch) -> AppSettings`
- `clear_cache() -> CacheReport`

Tauri events report import progress, overlay changes initiated by native windows, and external reopen requests. Inputs are validated once at the command boundary. Domain failures map to stable error codes plus Indonesian user messages.

## File ownership

Application data contains `lumadesk.sqlite3`, `media/`, `thumbnails/`, and `frames/`. Import writes to a temporary file, hashes and decodes it, moves it atomically into `media/`, then commits the database row. A scope guard removes temporary data if any stage fails.

The frontend receives asset URLs only for managed media and cache roots. It cannot request arbitrary local paths. Deletion runs as a coordinated operation: deactivate instances, delete the catalog record transactionally, then remove managed files. Failed file cleanup is logged and retried on startup.

## Rendering and windows

Rust reads animation metadata and decodes bounded frame batches. The overlay page renders frames to a transparent canvas using their original timing. A per-instance LRU budget prevents large animations from retaining unlimited decoded data.

Each overlay is an undecorated, transparent, always-on-top Tauri window. Its label is derived from an internal instance UUID. Display mode ignores cursor events; edit mode restores hit testing and focus. macOS builds enable the required private transparent-window API and are distributed directly rather than through the App Store.

## Migrations and compatibility

- SQL migrations are embedded in the core crate and run in one transaction at startup.
- Schema version is monotonic; downgrade is unsupported.
- Unknown newer schema versions fail closed with a backup instruction.
- Settings added in later versions receive explicit defaults.

## Release architecture

Pull requests and main-branch pushes run formatting, linting, type checks, tests, and platform builds. Tags matching `vMAJOR.MINOR.PATCH` run the same checks, package all targets, calculate SHA-256 files, and publish a GitHub Release using the matching CHANGELOG section. Publishing requires all platform jobs to succeed. Signing and notarization use protected repository secrets when configured.
