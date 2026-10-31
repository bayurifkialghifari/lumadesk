# Installation and releases

## Official packages

- Windows 10/11 x64: NSIS `.exe` installer.
- Windows 11 ARM64: NSIS `.exe` installer.
- macOS Apple Silicon: ARM64 `.dmg`.
- macOS Intel: x86-64 `.dmg`.
- Linux x86-64 and ARM64: `.AppImage`, Debian `.deb`, and RPM `.rpm`.

Linux runs overlays directly in X11 sessions. In Wayland sessions, LumaDesk uses XWayland for transparent always-on-top windows and pointer interaction.

## Release process

Releases follow Semantic Versioning. A `vMAJOR.MINOR.PATCH` Git tag builds every supported package. A release is published only after checks, tests, packaging, and native package inspection succeed for every platform. Each release includes SHA-256 checksums.

The tag, `package.json`, Cargo workspace, and Tauri configuration versions must match. The workflow rejects a release when any version differs. Linux verification inspects ELF architecture plus AppImage, Debian, and RPM contents; macOS verification checks the disk image, bundle identity, version, and executable architecture; Windows verification checks the PE architecture, product metadata, installer name, and updater signature.

The assembler accepts only the expected filenames and produces exactly 24 release assets: ten updater payloads, ten detached signatures, two DMGs, `latest.json`, and `SHA256SUMS.txt`. A tag run first creates a draft GitHub Release, downloads every uploaded asset into a clean directory, validates its inventory and checksums again, and publishes only after the downloaded draft matches the local assembly byte for byte. A manual run from `main` stops after storing the validated `release-ready` workflow artifact, making it a safe release dry run.

Manual GitHub Actions builds are downloaded as ZIP archives because GitHub wraps every workflow artifact in a ZIP. Extract the archive to access the installer inside. Tagged GitHub Releases expose each `.exe`, `.dmg`, `.AppImage`, `.deb`, and `.rpm` as a direct download.

## Automatic updates

Installed builds check the latest GitHub Release when the management window starts. When a newer version supports the current system and architecture, **Update vX.Y.Z** appears beside the window controls. Pressing it downloads the package, verifies its updater signature, installs it, and relaunches LumaDesk. See [Application updates](application-updates.md) for behavior and recovery details.

The release workflow signs the Windows NSIS installers and every Linux AppImage, Debian, and RPM package used by the in-app updater. macOS receives separate signed application bundle archives for Apple Silicon and Intel. `latest.json` maps each installed platform and package type to its matching payload and signature.

Updater signatures protect the in-app download path. Windows Authenticode signing and macOS signing and notarization are separate operating-system trust layers and will activate when the required certificates and repository secrets are configured.

Updater signing credentials are required for all release and standalone Windows builds. Optional Windows credentials must include both the certificate and password. Optional Apple code-signing and notarization credentials are also validated as complete groups; leaving those groups unset still produces updater-signed packages for testing and distribution.
