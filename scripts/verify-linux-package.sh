#!/usr/bin/env bash
set -euo pipefail

target="${1:?Rust target is required}"
architecture="${2:?Architecture is required}"
version="${3:?Version is required}"

case "$architecture" in
  x86_64)
    appimage_name="LumaDesk_${version}_amd64.AppImage"
    deb_name="LumaDesk_${version}_amd64.deb"
    rpm_name="LumaDesk-${version}-1.x86_64.rpm"
    deb_architecture="amd64"
    rpm_architecture="x86_64"
    file_pattern="x86-64"
    ;;
  aarch64)
    appimage_name="LumaDesk_${version}_aarch64.AppImage"
    deb_name="LumaDesk_${version}_arm64.deb"
    rpm_name="LumaDesk-${version}-1.aarch64.rpm"
    deb_architecture="arm64"
    rpm_architecture="aarch64"
    file_pattern="ARM aarch64"
    ;;
  *)
    echo "Unsupported Linux architecture: $architecture" >&2
    exit 1
    ;;
esac

bundle_directory="target/$target/release/bundle"
binary="target/$target/release/lumadesk"
appimage="$bundle_directory/appimage/$appimage_name"
deb="$bundle_directory/deb/$deb_name"
rpm_package="$bundle_directory/rpm/$rpm_name"

for payload in "$appimage" "$deb" "$rpm_package"; do
  [[ -s "$payload" ]] || { echo "Missing Linux package: $payload" >&2; exit 1; }
  [[ -s "$payload.sig" ]] || { echo "Missing updater signature: $payload.sig" >&2; exit 1; }
done
[[ -x "$binary" ]] || { echo "Missing packaged application binary: $binary" >&2; exit 1; }

verify_elf_architecture() {
  local path="$1"
  local description
  description="$(file -b "$path")"
  if [[ "$description" != *"ELF"* || "$description" != *"$file_pattern"* ]]; then
    echo "Expected $architecture ELF file at $path, found: $description" >&2
    exit 1
  fi
}

verify_elf_architecture "$binary"
verify_elf_architecture "$appimage"

temporary_directory="$(mktemp -d)"
cleanup() {
  rm -rf "$temporary_directory"
}
trap cleanup EXIT

(
  cd "$temporary_directory"
  "$OLDPWD/$appimage" --appimage-extract >/dev/null
)
appimage_binary="$temporary_directory/squashfs-root/usr/bin/lumadesk"
[[ -x "$appimage_binary" ]] || {
  echo "AppImage does not contain usr/bin/lumadesk." >&2
  exit 1
}
verify_elf_architecture "$appimage_binary"

deb_package_name="$(dpkg-deb -f "$deb" Package)"
deb_version="$(dpkg-deb -f "$deb" Version)"
actual_deb_architecture="$(dpkg-deb -f "$deb" Architecture)"
[[ "$deb_package_name" == "luma-desk" ]] || {
  echo "Expected Debian package luma-desk, found $deb_package_name." >&2
  exit 1
}
[[ "$deb_version" == "$version" ]] || {
  echo "Expected Debian version $version, found $deb_version." >&2
  exit 1
}
[[ "$actual_deb_architecture" == "$deb_architecture" ]] || {
  echo "Expected Debian architecture $deb_architecture, found $actual_deb_architecture." >&2
  exit 1
}
deb_root="$temporary_directory/deb"
dpkg-deb --extract "$deb" "$deb_root"
[[ -x "$deb_root/usr/bin/lumadesk" ]] || {
  echo "Debian package does not contain usr/bin/lumadesk." >&2
  exit 1
}
verify_elf_architecture "$deb_root/usr/bin/lumadesk"

rpm_package_name="$(rpm --query --package --queryformat '%{NAME}' "$rpm_package")"
rpm_version="$(rpm --query --package --queryformat '%{VERSION}' "$rpm_package")"
actual_rpm_architecture="$(rpm --query --package --queryformat '%{ARCH}' "$rpm_package")"
[[ "$rpm_package_name" == "luma-desk" ]] || {
  echo "Expected RPM package luma-desk, found $rpm_package_name." >&2
  exit 1
}
[[ "$rpm_version" == "$version" ]] || {
  echo "Expected RPM version $version, found $rpm_version." >&2
  exit 1
}
[[ "$actual_rpm_architecture" == "$rpm_architecture" ]] || {
  echo "Expected RPM architecture $rpm_architecture, found $actual_rpm_architecture." >&2
  exit 1
}
rpm_root="$temporary_directory/rpm"
mkdir -p "$rpm_root/usr/bin"
(
  cd "$rpm_root"
  rpm2cpio "$OLDPWD/$rpm_package" | cpio --extract --quiet ./usr/bin/lumadesk
)
[[ -x "$rpm_root/usr/bin/lumadesk" ]] || {
  echo "RPM package does not contain usr/bin/lumadesk." >&2
  exit 1
}
verify_elf_architecture "$rpm_root/usr/bin/lumadesk"

echo "Verified Linux $architecture packages for LumaDesk $version."
