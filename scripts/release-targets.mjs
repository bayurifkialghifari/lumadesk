export const RELEASE_TARGETS = Object.freeze([
  {
    artifactDirectory: "linux-x86_64",
    platform: "linux-x86_64-appimage",
    payloadName: (version) => `LumaDesk_${version}_amd64.AppImage`,
  },
  {
    artifactDirectory: "linux-x86_64",
    platform: "linux-x86_64-deb",
    payloadName: (version) => `LumaDesk_${version}_amd64.deb`,
  },
  {
    artifactDirectory: "linux-x86_64",
    platform: "linux-x86_64-rpm",
    payloadName: (version) => `LumaDesk-${version}-1.x86_64.rpm`,
  },
  {
    artifactDirectory: "linux-aarch64",
    platform: "linux-aarch64-appimage",
    payloadName: (version) => `LumaDesk_${version}_aarch64.AppImage`,
  },
  {
    artifactDirectory: "linux-aarch64",
    platform: "linux-aarch64-deb",
    payloadName: (version) => `LumaDesk_${version}_arm64.deb`,
  },
  {
    artifactDirectory: "linux-aarch64",
    platform: "linux-aarch64-rpm",
    payloadName: (version) => `LumaDesk-${version}-1.aarch64.rpm`,
  },
  {
    artifactDirectory: "macos-aarch64",
    platform: "darwin-aarch64-app",
    payloadName: () => "LumaDesk_aarch64.app.tar.gz",
  },
  {
    artifactDirectory: "macos-x86_64",
    platform: "darwin-x86_64-app",
    payloadName: () => "LumaDesk_x86_64.app.tar.gz",
  },
  {
    artifactDirectory: "windows-x86_64",
    platform: "windows-x86_64-nsis",
    payloadName: (version) => `LumaDesk_${version}_x64-setup.exe`,
  },
  {
    artifactDirectory: "windows-aarch64",
    platform: "windows-aarch64-nsis",
    payloadName: (version) => `LumaDesk_${version}_arm64-setup.exe`,
  },
]);

export const RELEASE_INSTALLERS = Object.freeze([
  {
    artifactDirectory: "macos-aarch64",
    filename: (version) => `LumaDesk_${version}_aarch64.dmg`,
  },
  {
    artifactDirectory: "macos-x86_64",
    filename: (version) => `LumaDesk_${version}_x64.dmg`,
  },
]);

export const EXPECTED_MANIFEST_TARGETS = RELEASE_TARGETS.length;
export const EXPECTED_RELEASE_ASSETS =
  RELEASE_TARGETS.length * 2 + RELEASE_INSTALLERS.length + 2;

export function expectedSourceAssets(version) {
  return [
    ...RELEASE_TARGETS.flatMap((target) => {
      const payload = target.payloadName(version);
      return [payload, `${payload}.sig`];
    }),
    ...RELEASE_INSTALLERS.map((installer) => installer.filename(version)),
  ];
}
