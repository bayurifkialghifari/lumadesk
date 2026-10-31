<picture>
  <img alt="LumaDesk — Your animation library, alive on the desktop." src="docs/assets/readme-banner.svg">
</picture>

<p align="center">
  Collect, shape, and place animated media anywhere on your desktop.
</p>

<p align="center">
  <a href="https://github.com/bayurifkialghifari/lumadesk/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/bayurifkialghifari/lumadesk/ci.yml?branch=main&style=flat-square&label=checks"></a>
  <a href="https://github.com/bayurifkialghifari/lumadesk/releases"><img alt="Release" src="https://img.shields.io/github/v/release/bayurifkialghifari/lumadesk?style=flat-square&display_name=tag"></a>
  <a href="LICENSE"><img alt="MIT License" src="https://img.shields.io/badge/license-MIT-39a8ff?style=flat-square"></a>
</p>

## A compact studio for desktop animation

LumaDesk combines a searchable media library, a non-destructive Creator Studio, and native transparent overlays in one cross-platform app. Import a GIF or image, prepare it once, then create as many independently controlled desktop instances as you want.

### Library

- Import GIF, APNG, WebP, PNG, and JPEG through the native picker or drag and drop.
- Search, filter, tag, favorite, rename, and safely remove managed media.
- Detect duplicate files by content and open external GIF discovery without an API key.

### Creator Studio

- Crop, rotate, mirror, and tune brightness, contrast, and saturation.
- Keep edits as reusable recipes while preserving the imported source.
- Choose instant edge removal or optional local Fast and Quality AI models.
- Turn still images into sprite animations with loop, once, and ping-pong playback.

### Desktop overlays

- Create multiple always-on-top instances with independent position, size, opacity, speed, flip, playback, and background settings.
- Drag an animation directly; its controls appear on press and hide when idle.
- Move overlays between connected displays while preserving their relative placement.
- Save complete desktop scenes, including visibility, playback, click-through, and monitor placement, then restore them in one action.
- Show, hide, play, pause, lock, or unlock every overlay from the manager or system tray.
- Restore the active layout after restart and keep overlays running from the system tray or menu bar.

### Settings and storage

- Control theme, Collection layout, startup behavior, overlay defaults, and local storage actions.
- Install or remove optional AI models and clear regenerable previews, frames, and masks.
- Store the catalog in SQLite and media inside the operating system's application-data directory.

## Download

| System | Direct release package |
| --- | --- |
| Windows 10/11 x64 | NSIS `.exe` installer |
| Windows 11 ARM64 | NSIS `.exe` installer |
| macOS Apple Silicon | ARM64 `.dmg` |
| macOS Intel | x86-64 `.dmg` |
| Linux x86-64 | `.AppImage`, `.deb`, `.rpm` |
| Linux ARM64 | `.AppImage`, `.deb`, `.rpm` |

Download tagged installers from [GitHub Releases](https://github.com/bayurifkialghifari/lumadesk/releases). GitHub Actions artifacts are ZIP containers; tagged release assets are the direct installer files.

## Development

Install the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your operating system, then run:

```bash
pnpm install
pnpm tauri dev
```

Run the primary checks with:

```bash
pnpm check
pnpm test
cargo test --workspace --locked
```

## Documentation

- [Animation library](docs/animation-library.md)
- [Importing media](docs/importing-media.md)
- [Discovering GIFs](docs/discovering-gifs.md)
- [Creator Studio](docs/creator-studio.md)
- [Sprite sheets](docs/sprite-sheets.md)
- [Desktop overlays](docs/desktop-overlays.md)
- [Desktop scenes](docs/desktop-scenes.md)
- [Multi-monitor layouts](docs/multi-monitor-layouts.md)
- [Click-through overlays](docs/click-through.md)
- [Background removal](docs/background-removal.md)
- [Settings and local data](docs/settings-and-local-data.md)
- [Window and application lifecycle](docs/window-lifecycle.md)
- [Installation and releases](docs/installation-and-releases.md)
- [Product requirements](PRD.md)
- [Design system](DESIGN.md)
- [Architecture](architecture.md)
- [Changelog](CHANGELOG.md)

## License

LumaDesk is available under the [MIT License](LICENSE). Redistributable starter media keeps its own license record.
