# Installation and releases

## Official packages

- Windows 10/11 x64: NSIS `.exe` installer.
- Windows 11 ARM64: NSIS `.exe` installer.
- macOS Apple Silicon: ARM64 `.dmg`.
- macOS Intel: x86-64 `.dmg`.
- Linux x86-64 and ARM64: `.AppImage`, Debian `.deb`, and RPM `.rpm`.

Linux runs overlays directly in X11 sessions. In Wayland sessions, LumaDesk uses XWayland for transparent always-on-top windows and pointer interaction.

## Release process

Releases follow Semantic Versioning. A `vMAJOR.MINOR.PATCH` Git tag builds every supported package. A release is published only after checks, tests, and packaging succeed for every platform. Each release includes SHA-256 checksums.

The tag, `package.json`, Cargo workspace, and Tauri configuration versions must match. The workflow rejects a release when any version differs. Linux, macOS, and Windows packages are collected first; the GitHub Release and `SHA256SUMS.txt` are published only after every build succeeds.

Manual GitHub Actions builds are downloaded as ZIP archives because GitHub wraps every workflow artifact in a ZIP. Extract the archive to access the installer inside. Tagged GitHub Releases expose each `.exe`, `.dmg`, `.AppImage`, `.deb`, and `.rpm` as a direct download.

## Automatic updates

Installed builds check the latest GitHub Release when the management window starts. When a newer version supports the current system and architecture, **Update vX.Y.Z** appears beside the window controls. Pressing it downloads the package, verifies its updater signature, installs it, and relaunches LumaDesk. See [Application updates](application-updates.md) for behavior and recovery details.

The release workflow signs the Windows NSIS installers and every Linux AppImage, Debian, and RPM package used by the in-app updater. macOS receives separate signed application bundle archives for Apple Silicon and Intel. `latest.json` maps each installed platform and package type to its matching payload and signature.

Updater signatures protect the in-app download path. Windows Authenticode signing and macOS signing and notarization are separate operating-system trust layers and will activate when the required certificates and repository secrets are configured.
