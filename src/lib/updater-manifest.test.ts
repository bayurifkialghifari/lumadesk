import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";

const temporaryDirectories: string[] = [];

afterEach(() => {
  for (const directory of temporaryDirectories.splice(0)) {
    rmSync(directory, { recursive: true, force: true });
  }
});

describe("updater manifest generator", () => {
  it("maps all release targets to signed GitHub assets", () => {
    const root = mkdtempSync(join(tmpdir(), "lumadesk-updater-"));
    temporaryDirectories.push(root);
    const artifacts = join(root, "artifacts");
    const output = join(root, "release", "latest.json");
    const fixtures = [
      ["linux-x86_64", "LumaDesk_0.6.1_amd64.AppImage"],
      ["linux-x86_64", "LumaDesk_0.6.1_amd64.deb"],
      ["linux-x86_64", "LumaDesk-0.6.1-1.x86_64.rpm"],
      ["linux-aarch64", "LumaDesk_0.6.1_arm64.AppImage"],
      ["linux-aarch64", "LumaDesk_0.6.1_arm64.deb"],
      ["linux-aarch64", "LumaDesk-0.6.1-1.aarch64.rpm"],
      ["macos-aarch64", "LumaDesk_aarch64.app.tar.gz"],
      ["macos-x86_64", "LumaDesk_x86_64.app.tar.gz"],
      ["windows-x86_64", "LumaDesk_0.6.1_x64-setup.exe"],
      ["windows-aarch64", "LumaDesk_0.6.1_arm64-setup.exe"],
    ] as const;

    for (const [directory, filename] of fixtures) {
      const destination = join(artifacts, directory, "bundle");
      mkdirSync(destination, { recursive: true });
      writeFileSync(join(destination, filename), "payload");
      writeFileSync(join(destination, `${filename}.sig`), `signature-${directory}\n`);
    }

    const script = new URL("../../scripts/create-updater-manifest.mjs", import.meta.url);
    execFileSync(
      process.execPath,
      [
        script.pathname,
        "--artifacts",
        artifacts,
        "--output",
        output,
        "--repository",
        "bayurifkialghifari/lumadesk",
        "--tag",
        "v0.6.1",
        "--version",
        "0.6.1",
        "--pub-date",
        "2026-10-31T00:00:00.000Z",
      ],
      { stdio: "pipe" },
    );

    const manifest = JSON.parse(readFileSync(output, "utf8"));
    expect(manifest).toMatchObject({
      version: "0.6.1",
      pub_date: "2026-10-31T00:00:00.000Z",
      platforms: {
        "linux-x86_64-appimage": { signature: "signature-linux-x86_64" },
        "linux-x86_64-deb": { signature: "signature-linux-x86_64" },
        "linux-x86_64-rpm": { signature: "signature-linux-x86_64" },
        "linux-aarch64-appimage": { signature: "signature-linux-aarch64" },
        "linux-aarch64-deb": { signature: "signature-linux-aarch64" },
        "linux-aarch64-rpm": { signature: "signature-linux-aarch64" },
        "darwin-aarch64-app": { signature: "signature-macos-aarch64" },
        "darwin-x86_64-app": { signature: "signature-macos-x86_64" },
        "windows-x86_64-nsis": { signature: "signature-windows-x86_64" },
        "windows-aarch64-nsis": { signature: "signature-windows-aarch64" },
      },
    });
    expect(manifest.platforms["windows-aarch64-nsis"].url).toBe(
      "https://github.com/bayurifkialghifari/lumadesk/releases/download/v0.6.1/LumaDesk_0.6.1_arm64-setup.exe",
    );
  });
});
