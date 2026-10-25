<picture>
  <img alt="LumaDesk — Your animation library, alive on the desktop." src="docs/assets/readme-banner.svg">
</picture>

<p align="center">
  Organize your GIFs, APNGs, WebPs, and images, then place them on your desktop as lightweight overlays.
</p>

<p align="center">
  <a href="https://github.com/bayurifkialghifari/lumadesk/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/bayurifkialghifari/lumadesk/ci.yml?branch=main&style=flat-square&label=checks"></a>
  <a href="https://github.com/bayurifkialghifari/lumadesk/releases"><img alt="Release" src="https://img.shields.io/github/v/release/bayurifkialghifari/lumadesk?style=flat-square&display_name=tag"></a>
  <a href="LICENSE"><img alt="MIT License" src="https://img.shields.io/badge/license-MIT-39a8ff?style=flat-square"></a>
</p>

## What you can do

- Import GIF, APNG, WebP, PNG, and JPEG files through the native picker or drag and drop.
- Search, tag, favorite, rename, and remove items from a local library.
- Create multiple transparent overlays with independent position, size, opacity, speed, and direction.
- Manage every active overlay, move it in edit mode, then play, pause, duplicate, or close each instance.
- Restore your overlay layout when the application starts again.
- Keep everything local, with no account and no telemetry.

## Download

| System | Package |
| --- | --- |
| Windows 10/11 x64 | NSIS `.exe` installer |
| Windows 11 ARM64 | NSIS `.exe` installer |
| macOS Apple Silicon | ARM64 `.dmg` |
| macOS Intel | x86-64 `.dmg` |
| Linux x86-64 | `.AppImage`, `.deb`, and `.rpm` |
| Linux ARM64 | `.AppImage`, `.deb`, and `.rpm` |

Installers will be available from [GitHub Releases](https://github.com/bayurifkialghifari/lumadesk/releases) with the first public release. Development builds can be run from source.

## Development

Install the native prerequisites listed by [Tauri 2](https://v2.tauri.app/start/prerequisites/) for your operating system, then run:

```bash
pnpm install
pnpm tauri dev
```

Run the primary checks with:

```bash
pnpm check
pnpm test
cargo test --workspace
```

## Documentation

- [Animation library](docs/koleksi.md)
- [Importing media](docs/impor-media.md)
- [Desktop overlays](docs/overlay-desktop.md)
- [Tray and application lifecycle](docs/tray-dan-lifecycle.md)
- [Settings and local data](docs/pengaturan-dan-data.md)
- [Installation and releases](docs/instalasi-dan-rilis.md)
- [Product requirements](PRD.md)
- [Design system](DESIGN.md)
- [Architecture](architecture.md)
- [Changelog](CHANGELOG.md)

## License

LumaDesk is available under the [MIT License](LICENSE). Third-party starter assets always include their own license records.
