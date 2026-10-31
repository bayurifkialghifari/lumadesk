# Release smoke tests

This checklist verifies the packages and update feed that users receive. The automated release workflow performs these checks before publication; maintainers can also run the workflow manually from `main` as a non-publishing dry run.

## Automated package checks

Each of the six build jobs must finish its native verifier before uploading artifacts:

- Windows x64 and ARM64: confirm the application PE architecture, product name, product version, exact NSIS installer filename, and non-empty updater signature. Require valid Authenticode signatures when optional Windows signing is enabled.
- macOS Apple Silicon and Intel: verify the DMG, mount it read-only, confirm one application bundle, inspect its identifier and version, and verify the executable architecture. Require code-signing and notarization checks when those optional credentials are enabled.
- Linux x86-64 and ARM64: inspect the application and AppImage ELF architecture, extract the AppImage, inspect Debian and RPM metadata, and confirm every package contains an executable `usr/bin/lumadesk`.

Every updater payload must have a non-empty detached `.sig` file. Updater signing credentials are mandatory even when operating-system trust signing is not configured.

## Assembly checks

The release assembly must contain exactly 24 flat files:

- 10 updater payloads: Windows NSIS for two architectures, macOS application archives for two architectures, and AppImage, Debian, and RPM packages for two Linux architectures;
- 10 matching detached signatures;
- 2 macOS DMGs;
- `latest.json`;
- `SHA256SUMS.txt`.

Validation rejects missing, duplicate, renamed, or unexpected assets. `latest.json` must contain exactly ten updater targets whose URLs point to the tagged GitHub Release and whose signatures match the detached files. `SHA256SUMS.txt` must cover every other release asset exactly once.

## Publication checks

For a tag run, automation uploads the validated files to a draft release, downloads the draft into a clean directory, revalidates all 24 files, compares every digest, and only then removes the draft flag. Never move an existing release tag to correct a failed release; fix the problem and publish a later patch version.

After publication, confirm that:

1. the release is public and contains exactly 24 assets;
2. Windows x64 and ARM64 installers download directly as `.exe` files;
3. both DMGs and all six Linux packages are present;
4. `latest.json` has ten platform mappings and reports the tagged version;
5. `SHA256SUMS.txt` validates all 23 other assets;
6. an installed older build shows the update action, completes a signed update, and relaunches;
7. a simulated failed download exposes retry and manual-download recovery actions.
