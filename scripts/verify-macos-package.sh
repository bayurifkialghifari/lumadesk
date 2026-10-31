#!/usr/bin/env bash
set -euo pipefail

target="${1:?Rust target is required}"
architecture="${2:?Architecture is required}"
version="${3:?Version is required}"
require_codesign="${4:-false}"
require_notarization="${5:-false}"

case "$architecture" in
  aarch64)
    archive_name="LumaDesk_aarch64.app.tar.gz"
    dmg_name="LumaDesk_${version}_aarch64.dmg"
    expected_architecture="arm64"
    ;;
  x86_64)
    archive_name="LumaDesk_x86_64.app.tar.gz"
    dmg_name="LumaDesk_${version}_x64.dmg"
    expected_architecture="x86_64"
    ;;
  *)
    echo "Unsupported macOS architecture: $architecture" >&2
    exit 1
    ;;
esac

bundle_directory="target/$target/release/bundle"
archive="$bundle_directory/macos/$archive_name"
dmg="$bundle_directory/dmg/$dmg_name"
[[ -s "$archive" ]] || { echo "Missing macOS updater archive: $archive" >&2; exit 1; }
[[ -s "$archive.sig" ]] || { echo "Missing macOS updater signature: $archive.sig" >&2; exit 1; }
[[ -s "$dmg" ]] || { echo "Missing macOS disk image: $dmg" >&2; exit 1; }

hdiutil verify "$dmg"
mount_point="$(mktemp -d)"
cleanup() {
  hdiutil detach "$mount_point" -quiet >/dev/null 2>&1 || true
  rm -rf "$mount_point"
}
trap cleanup EXIT
hdiutil attach "$dmg" -readonly -nobrowse -mountpoint "$mount_point" -quiet

applications=()
while IFS= read -r application; do
  applications+=("$application")
done < <(find "$mount_point" -maxdepth 2 -type d -name '*.app')
if [[ "${#applications[@]}" -ne 1 ]]; then
  echo "Expected exactly one app bundle in $dmg, found ${#applications[@]}." >&2
  exit 1
fi
app="${applications[0]}"
plist="$app/Contents/Info.plist"
identifier="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$plist")"
bundle_version="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$plist")"
executable_name="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$plist")"
executable="$app/Contents/MacOS/$executable_name"

[[ "$identifier" == "app.lumadesk.desktop" ]] || {
  echo "Expected bundle identifier app.lumadesk.desktop, found $identifier." >&2
  exit 1
}
[[ "$bundle_version" == "$version" ]] || {
  echo "Expected bundle version $version, found $bundle_version." >&2
  exit 1
}
[[ -x "$executable" ]] || { echo "Missing packaged executable: $executable" >&2; exit 1; }
architectures="$(lipo -archs "$executable")"
[[ " $architectures " == *" $expected_architecture "* ]] || {
  echo "Expected $expected_architecture executable, found $architectures." >&2
  exit 1
}

if [[ "$require_codesign" == "true" ]]; then
  codesign --verify --deep --strict --verbose=2 "$app"
fi
if [[ "$require_notarization" == "true" ]]; then
  xcrun stapler validate "$dmg"
  spctl --assess --type execute --verbose=2 "$app"
fi

echo "Verified macOS $architecture package for LumaDesk $version."
