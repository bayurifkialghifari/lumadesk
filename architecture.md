# LumaDesk Architecture

## System shape

LumaDesk is a Tauri 2 desktop application. Svelte 5 renders the management window and overlay control surfaces. Rust owns trusted boundaries: files, SQLite, media decoding, edit validation, model management, generated caches, application lifecycle, and native windows.

```mermaid
flowchart LR
  UI[Svelte management UI] -->|typed Tauri commands| APP[Rust application layer]
  APP --> DB[(SQLite catalog)]
  APP --> MEDIA[Managed media store]
  APP --> CACHE[Frame and mask caches]
  APP --> AI[Optional ONNX models]
  APP --> WINDOWS[Native overlay windows]
  WINDOWS --> RENDER[Canvas recipe renderer]
  APP --> LIFECYCLE[Tray and window lifecycle]
  UI -->|check, install, relaunch| UPDATER[Tauri updater and process plugins]
  UPDATER -->|signed manifest and package| RELEASES[GitHub Releases]
  UI -->|typed Gallery commands| GALLERY[Verified Gallery cache]
  GALLERY -->|HTTPS catalog and assets| STATIC[Static GitHub catalog]
```

## Repository boundaries

- `src/`: Svelte application, runtime schemas, typed command client, controllers, pages, components, and CSS tokens.
- `src-tauri/src/`: Tauri commands, native window coordination, local model inference, single-instance behavior, and tray lifecycle.
- `crates/lumadesk-core/`: platform-independent domain types, SQLite repositories, import pipeline, edit validation, frame decoding, and generated-media records.
- `gallery/`: versioned catalog, contribution instructions, and starter-media license record.
- `src/assets/gallery/`: original redistributable starter media bundled for offline preview and import.
- `docs/`: one English guide per user-facing feature.

The core crate has no Tauri dependency. Database, editing, importing, sprite decoding, and cache behavior run in tests without a graphical session.

## Data model

### `media_items`

Each item stores its UUID, unique SHA-256 content hash, display name, managed path, asset kind, media format, dimensions, frame metadata, favorite state, and timestamps. Tags use `tags` and `media_tags`. The backend resolves managed paths from item IDs; arbitrary frontend paths are never accepted for catalog operations.

### `edit_recipes`

One recipe belongs to each media item. It stores an optional crop, quarter-turn rotation, horizontal flip, background-removal mode, brightness, contrast, saturation, and update timestamp. The recipe describes presentation and never rewrites the source file.

### `sprite_sequences`

An optional row stores columns, rows, frame count, frame duration, and loop, once, or ping-pong playback. Setting a sequence changes the item's asset kind to `sprite_sheet`; clearing it returns the item to its source-derived kind.

### `media_variants`

Generated assets are registered with a media ID, variant kind, validated relative path, dimensions, and creation time. AI masks use the `segmentation_mask` kind. The table lets cache cleanup, media deletion, sprite changes, and model removal delete both files and database records consistently.

### `overlay_instances`

Each instance references a media item and stores logical position, monitor identity, width, opacity, speed, horizontal flip, immediate edge-removal state, click-through state, playback state, z-order, and active state. Logical coordinates avoid DPI-dependent persistence. The media recipe and sprite sequence apply in addition to these per-instance controls.

### `desktop_scenes` and `desktop_scene_items`

A scene owns a unique case-insensitive name and an ordered snapshot of the complete persisted overlay layout. Each item stores media identity, saved monitor name and dimensions, normalized position, size, opacity, speed, flip, background removal, playback, click-through, z-order, and visibility. Deleting media removes dependent scene items while leaving the scene itself available for review or update.

### `settings`

A key/value table stores theme, startup behavior, overlay restoration, close-to-tray behavior, Collection view, and overlay defaults. The application layer parses and validates the complete typed settings object before exposing it.

## Creator save transaction

`save_creator_changes` validates the recipe and the proposed sprite sequence against the same effective frame dimensions. It then writes both records in one SQLite transaction. Invalid sprite metadata or an out-of-bounds crop cannot leave a partially updated recipe.

After a successful sprite change, the application removes decoded frames and segmentation variants whose dimensions or source key are now stale, then emits an update to active overlay windows. Generated-media operations share an async lock so Creator saves, model removal, mask generation, media deletion, and cache cleanup do not race over the same files.

## Command surface

The frontend calls serializable commands whose responses are checked again with Zod schemas. Major groups include:

- catalog: list, import, rename, favorite, tag, remove, resolve asset URL, and get frame batches;
- Creator: get or update recipes, atomically save Creator changes, and get, set, or clear sprite sequences;
- overlays: list, create, show, move, move between monitors, update, duplicate, remove, patch all, and close all;
- scenes: list, create, update, rename, duplicate, delete, and apply complete desktop snapshots;
- settings: get, update, open the data directory, and clear generated caches;
- local AI: list model status, download, remove, and request a bounded mask batch.
- Gallery: fetch the catalog, cache a verified preview, import a verified catalog item, and import supported media from an HTTPS URL.

Inputs are validated at the command boundary. Domain failures map to stable error codes with English messages. If a native overlay update fails, persisted instance state is rolled back. Overlay and media removal close dependent windows before deleting records and restore captured windows when persistence fails.

Monitor descriptors come from Tauri in logical coordinates. Moving one overlay preserves its normalized position from the source display, maps it to the selected display, and clamps its rendered bounds. Scene application resolves a saved monitor by name and falls back to the primary display, or the first available display when no primary is reported.

Applying a scene first prepares all media and target coordinates, replaces the persisted overlay layout transactionally, and then synchronizes native windows. A native failure restores both the previous database layout and its windows. Global show, hide, play, pause, lock, and unlock operations use the same persistence-first synchronization with rollback.

## File ownership

Application data contains `lumadesk.sqlite3`, `media/`, `thumbnails/`, `frames/`, `masks/`, `models/`, and `gallery/`. Import writes to a temporary file, hashes and decodes it, moves it atomically into `media/`, then commits the catalog row. A scope guard removes temporary data if any stage fails.

Model downloads use a `.part` file and become available only after exact length and SHA-256 verification. Fast and Quality model files remain optional and removable. Clearing generated cache removes previews, decoded frames, and masks while preserving the database, managed source media, settings, and installed models.

The frontend receives asset URLs only for approved media and cache roots. Deletion quarantines a managed file before the database transaction, restores it on failure, and clears stale quarantine files at startup.

## Community Gallery architecture

The Community Gallery has no dedicated backend. `gallery/catalog.v1.json` is published with the repository and fetched from GitHub over HTTPS. Rust parses a strict schema and validates IDs, text bounds, tags, supported formats, URLs, declared sizes, and SHA-256 values. A fresh disk cache is preferred for normal launches; a refresh requests the network copy. Failure falls back to any valid cache and then to the catalog embedded in the binary.

Preview and download files pass through the Rust network layer to avoid browser CORS and filesystem access. Downloads use bounded streaming and temporary files, require a final HTTPS URL, and become visible only after exact size and SHA-256 verification. The four starter assets are embedded in the binary and satisfy matching catalog hashes without a network request. Approved previews live under the scoped `gallery/` application-data directory.

Gallery import stores the declared title and tags in the same SQLite transaction as the new media row and edit recipe. Existing content hashes return the normal duplicate outcome. Direct URL import accepts HTTPS only, streams at most 100 MiB into a temporary file, and then invokes the same media inspection, managed-copy, hashing, and duplicate pipeline as local files.

CI runs `scripts/validate-gallery.mjs` before frontend and Rust checks. It rejects schema drift, duplicate IDs, insecure URLs, missing local assets, and byte-size or SHA-256 mismatches. New entries are therefore reviewed and released like source code. Accounts, public uploads, ratings, and moderation would require a service and remain outside this static-catalog design.

## Rendering and playback

Rust generates a persistent frame cache atomically. The frontend requests bounded frame batches, prefetches near a batch boundary, and keeps only the current and next batch in memory. Static images render once without a scheduler. Sprite sheets are decoded cell by cell according to their saved sequence and expose normalized playback frames for the same player.

The preview and overlay renderers apply crop, rotation, recipe flip, color filters, and the selected background mode. Loop, once, and ping-pong sequences share the same timing controller. The per-instance flip combines with the recipe flip without changing stored media.

Edge removal flood-fills similar pixels connected to frame boundaries and feathers their alpha. AI modes request masks generated by U²-NetP or U²-Net through `tract-onnx`. Masks are keyed by media content and sprite metadata, stored as regenerable PNG variants, and composited through a bounded canvas cache.

## Native windows and lifecycle

Each visible overlay is an undecorated, transparent, always-on-top Tauri window whose label derives from an internal instance UUID. Pressing the animation reveals its compact toolbar and boundary. Native dragging starts from the animation or drag handle, and a debounced serial writer persists logical position. Click-through disables focus and ignores cursor events at the native-window layer.

The management window uses a custom title bar with Minimize, Maximize/Restore, and Close. At startup, the updater plugin reads the signed release manifest from GitHub. A newer version replaces the quiet version label with an install action. The frontend reports download progress, while the native plugin downloads, verifies, and installs the platform package before the process plugin relaunches LumaDesk. Failed checks remain quiet for offline use. Download and installation failures retain the discovered release so the user can retry or open its GitHub Release manually. If relaunching fails after installation, the controller preserves a restart-required state so the process can be relaunched without downloading the package again. Lifecycle tokens prevent late asynchronous results from overwriting a newer updater state.

Closing hides the management window to the tray or menu bar when `close_to_tray` is enabled. Tray actions restore the window, show or hide all overlays, play or pause all overlays, lock or unlock click-through, or quit the process. Single-instance launches use the same restore path. Overlay windows bypass the main-window close handler.

## Migrations and compatibility

- SQL migrations are embedded in the core crate and run in one transaction at startup.
- Schema versions increase monotonically; downgrade is unsupported.
- A newer unknown schema fails closed with a backup instruction.
- Settings introduced in later versions receive explicit defaults.

## Release architecture

Pushes and pull requests run frontend types, tests, lint, build, Rust formatting, workspace tests, and Clippy. Tags matching `vMAJOR.MINOR.PATCH` build Windows x86-64 and ARM64 NSIS installers, macOS Apple Silicon and Intel DMGs, and Linux x86-64 and ARM64 AppImage, Debian, and RPM packages. Each matrix job creates signed native updater payloads using the mandatory protected updater key, then verifies package names, target architecture, application identity, version, and detached signatures with native platform tools. A manual workflow run from `main` performs the same six builds and assembly checks without publishing.

Publishing starts only after every platform job succeeds. The assembler preserves each matrix directory while resolving updater targets, rejects duplicate or unexpected flattened names, generates `latest.json` with ten installer-specific entries and GitHub download URLs, then creates `SHA256SUMS.txt`. The resulting contract is exactly 24 files: ten updater payloads, ten detached signatures, two DMGs, the manifest, and the checksum file. Tag builds upload these files to a draft release, download them again, compare every digest with the validated local assembly, and publish only after that second validation succeeds. Linux AppImage, Debian, and RPM installations resolve to their matching package type. The public key embedded in the app verifies updater packages; the private key exists only in GitHub Actions secrets. Windows Authenticode signing, Apple code signing, and Apple notarization are optional all-or-none credential groups and do not block updater-signed builds when absent.
