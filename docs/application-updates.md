# Application updates

LumaDesk can install new versions without sending the user back through the manual download flow. The updater supports Windows x64 and ARM64, macOS Apple Silicon and Intel, and Linux x86-64 and ARM64.

## Checking for an update

The management window checks the latest GitHub Release once during startup. The check runs in the background and leaves the current version visible in the title bar when no newer release exists. A failed check does not interrupt startup or create a notification, so LumaDesk continues normally while offline.

When a newer compatible release exists, the version label becomes **Update vX.Y.Z**. LumaDesk never starts installation from the background check; the user must press this action.

## Downloading and installing

After the update action is pressed, the title bar shows download progress when the server provides a package size. An activity state is shown when the total size is unknown. The native updater verifies the downloaded archive with the public key embedded in LumaDesk before installing it. A package with a missing or invalid signature is rejected.

LumaDesk changes the title-bar status to **Installing** after the download completes. Successful installation relaunches the application into the new version. If downloading or installation fails, the update action becomes available again and an English error toast asks the user to retry. If installation succeeds but the operating system prevents the relaunch, the title bar shows **Restart to finish update** so the relaunch can be retried without downloading the package again.

## Release source

The update feed is the `latest.json` asset attached to the latest GitHub Release. It contains the version, publication date, release-note reference, and a signed package URL for each supported target:

| Update target | Package |
| --- | --- |
| Windows x64 | NSIS updater archive |
| Windows ARM64 | NSIS updater archive |
| macOS Apple Silicon | Application bundle archive |
| macOS Intel | Application bundle archive |
| Linux x86-64 | AppImage updater archive |
| Linux ARM64 | AppImage updater archive |

The normal `.exe`, `.dmg`, `.AppImage`, `.deb`, and `.rpm` assets remain available for first-time installation and manual recovery. Updater archives and their detached signatures are release-system files and do not replace those installers.

## Version behavior

LumaDesk compares the installed semantic version with the release version. It offers only a newer compatible release. The package version, Cargo workspace version, Tauri version, title-bar version, Git tag, release title, and update manifest version are kept in sync by the release process.

The first build that contains this feature must still be installed manually. Automatic updates apply from that build to later releases.
