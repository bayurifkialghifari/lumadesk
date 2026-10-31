import { readFile, readdir } from "node:fs/promises";
import { basename, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import {
  EXPECTED_MANIFEST_TARGETS,
  EXPECTED_RELEASE_ASSETS,
  RELEASE_INSTALLERS,
  RELEASE_TARGETS,
  expectedSourceAssets,
} from "./release-targets.mjs";
import { requiredArgument, sha256, validateReleaseIdentity } from "./release-utils.mjs";

function sameInventory(actual, expected) {
  return JSON.stringify([...actual].sort()) === JSON.stringify([...expected].sort());
}

async function validateChecksums(directory, expectedAssets) {
  const checksumPath = join(directory, "SHA256SUMS.txt");
  const lines = (await readFile(checksumPath, "utf8"))
    .split("\n")
    .filter((line) => line.length > 0);
  const checksums = new Map();
  for (const line of lines) {
    const match = /^([a-f0-9]{64})  (.+)$/.exec(line);
    if (match === null) throw new Error(`Invalid checksum line: ${line}.`);
    const [, digest, filename] = match;
    if (checksums.has(filename)) throw new Error(`Duplicate checksum entry for ${filename}.`);
    checksums.set(filename, digest);
  }

  const coveredAssets = expectedAssets.filter((name) => name !== "SHA256SUMS.txt");
  if (!sameInventory(checksums.keys(), coveredAssets)) {
    throw new Error("Checksum inventory does not exactly cover the release files.");
  }
  for (const [filename, digest] of checksums) {
    const actual = await sha256(join(directory, filename));
    if (actual !== digest) throw new Error(`Checksum mismatch for ${filename}.`);
  }
}

export async function validateReleaseDirectory({ directory, repository, tag, version }) {
  validateReleaseIdentity({ repository, tag, version });
  const entries = await readdir(directory, { withFileTypes: true });
  if (entries.some((entry) => !entry.isFile())) {
    throw new Error("Release directory must contain only flat files.");
  }
  const actualAssets = entries.map((entry) => entry.name);
  const expectedAssets = [
    ...expectedSourceAssets(version),
    "latest.json",
    "SHA256SUMS.txt",
  ];
  if (expectedAssets.length !== EXPECTED_RELEASE_ASSETS) {
    throw new Error("Internal release asset count is inconsistent.");
  }
  if (!sameInventory(actualAssets, expectedAssets)) {
    throw new Error(
      `Release asset inventory mismatch. Expected exactly ${EXPECTED_RELEASE_ASSETS} assets.`,
    );
  }

  const manifest = JSON.parse(await readFile(join(directory, "latest.json"), "utf8"));
  if (manifest.version !== version) {
    throw new Error(
      `Updater manifest version ${manifest.version} does not match release version ${version}.`,
    );
  }
  if (Number.isNaN(Date.parse(manifest.pub_date))) {
    throw new Error("Updater manifest pub_date is invalid.");
  }
  const platforms = manifest.platforms;
  if (platforms === null || typeof platforms !== "object") {
    throw new Error("Updater manifest platforms must be an object.");
  }
  const platformNames = Object.keys(platforms);
  if (
    platformNames.length !== EXPECTED_MANIFEST_TARGETS ||
    !sameInventory(
      platformNames,
      RELEASE_TARGETS.map((target) => target.platform),
    )
  ) {
    throw new Error(`Updater manifest must contain exactly ${EXPECTED_MANIFEST_TARGETS} targets.`);
  }

  const payloadNames = new Set();
  for (const target of RELEASE_TARGETS) {
    const payloadName = target.payloadName(version);
    const signatureName = `${payloadName}.sig`;
    const platform = platforms[target.platform];
    const expectedUrl = `https://github.com/${repository}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(payloadName)}`;
    if (platform.url !== expectedUrl) {
      throw new Error(`Updater URL for ${target.platform} does not match ${expectedUrl}.`);
    }
    if (payloadNames.has(payloadName)) {
      throw new Error(`Duplicate updater payload filename ${payloadName}.`);
    }
    payloadNames.add(payloadName);
    const signature = (await readFile(join(directory, signatureName), "utf8")).trim();
    if (signature.length === 0) throw new Error(`Updater signature ${signatureName} is empty.`);
    if (platform.signature !== signature) {
      throw new Error(`Updater signature for ${target.platform} does not match ${signatureName}.`);
    }
  }

  for (const installer of RELEASE_INSTALLERS) {
    const filename = installer.filename(version);
    if (!actualAssets.includes(filename)) throw new Error(`Missing installer ${filename}.`);
  }

  await validateChecksums(directory, expectedAssets);
  return expectedAssets;
}

async function compareDirectories(sourceDirectory, comparisonDirectory, assets) {
  const comparisonEntries = await readdir(comparisonDirectory, { withFileTypes: true });
  if (
    comparisonEntries.some((entry) => !entry.isFile()) ||
    !sameInventory(
      comparisonEntries.map((entry) => entry.name),
      assets,
    )
  ) {
    throw new Error("Draft release assets do not match the validated local inventory.");
  }
  for (const filename of assets) {
    const [sourceDigest, comparisonDigest] = await Promise.all([
      sha256(join(sourceDirectory, filename)),
      sha256(join(comparisonDirectory, filename)),
    ]);
    if (sourceDigest !== comparisonDigest) {
      throw new Error(`Draft release asset ${basename(filename)} differs from the validated file.`);
    }
  }
}

async function main() {
  const args = process.argv.slice(2);
  const directory = resolve(requiredArgument(args, "--directory"));
  const repository = requiredArgument(args, "--repository");
  const tag = requiredArgument(args, "--tag");
  const version = requiredArgument(args, "--version");
  const assets = await validateReleaseDirectory({ directory, repository, tag, version });
  if (args.includes("--compare")) {
    const comparisonDirectory = resolve(requiredArgument(args, "--compare"));
    await validateReleaseDirectory({ directory: comparisonDirectory, repository, tag, version });
    await compareDirectories(directory, comparisonDirectory, assets);
  }
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  });
}
