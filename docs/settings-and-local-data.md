# Settings and local data

Settings are grouped by appearance, desktop behavior, overlay defaults, storage, and local AI models. Changes are applied when **Save settings** is selected.

## Appearance

- **Theme:** use the LumaDesk dark theme or follow the operating-system color preference.
- **Collection view:** choose the visual grid or compact list used by the library.

## Desktop behavior

- **Launch at login:** start LumaDesk after the user signs in; disabled by default.
- **Close to tray:** keep overlays running after the management window closes; enabled by default.
- **Restore overlays:** reopen the last active desktop layout at launch.

## Overlay defaults

Width, opacity, and playback speed become the starting values for new desktop instances. Existing instances keep their own saved values.

## Storage

**Clear cache** removes generated previews, decoded frames, and background masks. Collection records, tags, edit recipes, sprite settings, managed media, overlays, desktop scenes, application settings, and installed AI models remain intact.

**Open data folder** opens the operating system's application-data directory containing:

- `lumadesk.sqlite3` for the catalog, tags, recipes, variants, settings, overlay instances, and desktop scenes;
- `media/` for managed copies of imported files;
- `thumbnails/`, `frames/`, and `masks/` for regenerable output;
- `models/` for optional downloaded ONNX models.

## Local AI models

Edge removal is built in. Fast and Quality AI models can be installed or removed independently. Every download is verified before use. Removing a model deletes that model file and its generated masks while preserving source media and Creator recipes.
