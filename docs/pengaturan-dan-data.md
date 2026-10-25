# Settings and local data

## General settings

- **Jalankan saat masuk** (Launch at login): starts LumaDesk after the user signs in; disabled by default.
- **Pulihkan susunan** (Restore layout): activates previously active overlays when the application starts.
- **Collection view:** grid or list.
- **Overlay defaults:** size, opacity, and speed for new instances.
- **Cache limit:** bounds the storage used by thumbnails and decoded frames.

## Cache

**Bersihkan cache** (Clear cache) removes thumbnails and decoded frames that LumaDesk can regenerate. Collection items, tags, settings, managed media, and overlay layouts remain intact.

## Data location and contents

**Buka folder data** (Open data folder) opens the application directory containing:

- the SQLite database for metadata, tags, settings, and layouts;
- the `media` directory for managed copies of imported files;
- the `thumbnails` and `frames` cache directories.

LumaDesk does not require an account, cloud storage, analytics, or telemetry for its core features.
