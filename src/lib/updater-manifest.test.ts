import { spawnSync } from "node:child_process";
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";

const VERSION = "9.8.7";
const REPOSITORY = "bayurifkialghifari/lumadesk";
const fixtures = [
  ["linux-x86_64", `LumaDesk_${VERSION}_amd64.AppImage`],
  ["linux-x86_64", `LumaDesk_${VERSION}_amd64.deb`],
  ["linux-x86_64", `LumaDesk-${VERSION}-1.x86_64.rpm`],
  ["linux-aarch64", `LumaDesk_${VERSION}_aarch64.AppImage`],
  ["linux-aarch64", `LumaDesk_${VERSION}_arm64.deb`],
  ["linux-aarch64", `LumaDesk-${VERSION}-1.aarch64.rpm`],
  ["macos-aarch64", "LumaDesk_aarch64.app.tar.gz"],
  ["macos-x86_64", "LumaDesk_x86_64.app.tar.gz"],
  ["windows-x86_64", `LumaDesk_${VERSION}_x64-setup.exe`],
  ["windows-aarch64", `LumaDesk_${VERSION}_arm64-setup.exe`],
] as const;
const installers = [
  ["macos-aarch64", `LumaDesk_${VERSION}_aarch64.dmg`],
  ["macos-x86_64", `LumaDesk_${VERSION}_x64.dmg`],
] as const;
const temporaryDirectories: string[] = [];
const assembleScript = new URL("../../scripts/assemble-release.mjs", import.meta.url).pathname;
const validateScript = new URL("../../scripts/validate-release.mjs", import.meta.url).pathname;

afterEach(() => {
  for (const directory of temporaryDirectories.splice(0)) {
    rmSync(directory, { recursive: true, force: true });
  }
});

function createFixture() {
  const root = mkdtempSync(join(tmpdir(), "lumadesk-release-"));
  temporaryDirectories.push(root);
  const artifacts = join(root, "artifacts");
  const release = join(root, "release");

  for (const [directory, filename] of fixtures) {
    const destination = join(artifacts, directory, "bundle");
    mkdirSync(destination, { recursive: true });
    writeFileSync(join(destination, filename), `payload-${directory}-${filename}`);
    writeFileSync(join(destination, `${filename}.sig`), `signature-${directory}-${filename}\n`);
  }
  for (const [directory, filename] of installers) {
    const destination = join(artifacts, directory, "bundle");
    mkdirSync(destination, { recursive: true });
    writeFileSync(join(destination, filename), `installer-${directory}`);
  }
  return { root, artifacts, release };
}

function runScript(script: string, args: string[]) {
  return spawnSync(process.execPath, [script, ...args], { encoding: "utf8" });
}

function assemble(artifacts: string, release: string) {
  const result = runScript(assembleScript, [
    "--artifacts",
    artifacts,
    "--output",
    release,
    "--repository",
    REPOSITORY,
    "--tag",
    `v${VERSION}`,
    "--version",
    VERSION,
    "--pub-date",
    "2026-10-31T00:00:00.000Z",
  ]);
  expect(result.stderr).toBe("");
  expect(result.status).toBe(0);
}

function validate(release: string, extraArgs: string[] = []) {
  return runScript(validateScript, [
    "--directory",
    release,
    "--repository",
    REPOSITORY,
    "--tag",
    `v${VERSION}`,
    "--version",
    VERSION,
    ...extraArgs,
  ]);
}

describe("release assembly and validation", () => {
  it("builds and validates the complete 24-asset release", () => {
    const { root, artifacts, release } = createFixture();
    assemble(artifacts, release);
    const downloadedDraft = join(root, "draft");
    cpSync(release, downloadedDraft, { recursive: true });

    const result = validate(release, ["--compare", downloadedDraft]);
    const manifest = JSON.parse(readFileSync(join(release, "latest.json"), "utf8"));

    expect(result.stderr).toBe("");
    expect(result.status).toBe(0);
    expect(Object.keys(manifest.platforms)).toHaveLength(10);
    expect(manifest.platforms["windows-aarch64-nsis"].url).toBe(
      `https://github.com/${REPOSITORY}/releases/download/v${VERSION}/LumaDesk_${VERSION}_arm64-setup.exe`,
    );
    expect(readFileSync(join(release, "SHA256SUMS.txt"), "utf8").trim().split("\n")).toHaveLength(
      23,
    );
  });

  it("rejects a missing updater signature", () => {
    const { artifacts, release } = createFixture();
    assemble(artifacts, release);
    unlinkSync(join(release, `LumaDesk_${VERSION}_arm64-setup.exe.sig`));

    const result = validate(release);

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain("Release asset inventory mismatch");
  });

  it("rejects duplicate flattened payload names", () => {
    const { artifacts, release } = createFixture();
    const duplicateDirectory = join(artifacts, "linux-x86_64", "duplicate");
    mkdirSync(duplicateDirectory, { recursive: true });
    writeFileSync(join(duplicateDirectory, `LumaDesk_${VERSION}_amd64.AppImage`), "duplicate");

    const result = runScript(assembleScript, [
      "--artifacts",
      artifacts,
      "--output",
      release,
      "--repository",
      REPOSITORY,
      "--tag",
      `v${VERSION}`,
      "--version",
      VERSION,
    ]);

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain("duplicate flattened filenames");
  });

  it("rejects an updater URL that does not match the release payload", () => {
    const { artifacts, release } = createFixture();
    assemble(artifacts, release);
    const manifestPath = join(release, "latest.json");
    const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    manifest.platforms["windows-aarch64-nsis"].url = "https://example.com/wrong.exe";
    writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);

    const result = validate(release);

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain("Updater URL for windows-aarch64-nsis does not match");
  });

  it("rejects an unexpected release asset", () => {
    const { artifacts, release } = createFixture();
    assemble(artifacts, release);
    writeFileSync(join(release, "unexpected.zip"), "unexpected");

    const result = validate(release);

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain("Release asset inventory mismatch");
  });

  it("rejects an incorrect version and tag pair", () => {
    const { artifacts, release } = createFixture();
    const result = runScript(assembleScript, [
      "--artifacts",
      artifacts,
      "--output",
      release,
      "--repository",
      REPOSITORY,
      "--tag",
      "v9.8.8",
      "--version",
      VERSION,
    ]);

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain(`Tag v9.8.8 does not match version ${VERSION}`);
  });

  it("rejects a payload modified after checksums were generated", () => {
    const { artifacts, release } = createFixture();
    assemble(artifacts, release);
    writeFileSync(join(release, `LumaDesk_${VERSION}_arm64-setup.exe`), "modified");

    const result = validate(release);

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain(`Checksum mismatch for LumaDesk_${VERSION}_arm64-setup.exe`);
  });
});
