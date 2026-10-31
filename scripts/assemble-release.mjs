import { copyFile, mkdir, readdir, rm, writeFile } from "node:fs/promises";
import { basename, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { createUpdaterManifest } from "./create-updater-manifest.mjs";
import {
  RELEASE_INSTALLERS,
  RELEASE_TARGETS,
  expectedSourceAssets,
} from "./release-targets.mjs";
import {
  listFiles,
  requiredArgument,
  sha256,
  validateReleaseIdentity,
} from "./release-utils.mjs";

async function findExpectedFile(artifactsDirectory, artifactDirectory, filename) {
  const files = await listFiles(join(artifactsDirectory, artifactDirectory));
  const matches = files.filter((file) => basename(file) === filename);
  if (matches.length !== 1) {
    throw new Error(
      `Expected exactly one ${filename} in ${artifactDirectory}, found ${matches.length}.`,
    );
  }
  return matches[0];
}

export async function assembleRelease({
  artifactsDirectory,
  outputDirectory,
  repository,
  tag,
  version,
  pubDate,
}) {
  validateReleaseIdentity({ repository, tag, version });
  const expectedNames = expectedSourceAssets(version).sort();
  const artifactFiles = await listFiles(artifactsDirectory);
  const actualNames = artifactFiles.map((file) => basename(file)).sort();
  if (new Set(actualNames).size !== actualNames.length) {
    throw new Error("Release artifacts contain duplicate flattened filenames.");
  }
  if (JSON.stringify(actualNames) !== JSON.stringify(expectedNames)) {
    throw new Error(
      `Release artifact inventory mismatch. Expected ${expectedNames.join(", ")}; received ${actualNames.join(", ")}.`,
    );
  }

  await rm(outputDirectory, { recursive: true, force: true });
  await mkdir(outputDirectory, { recursive: true });

  for (const target of RELEASE_TARGETS) {
    const payloadName = target.payloadName(version);
    const payload = await findExpectedFile(
      artifactsDirectory,
      target.artifactDirectory,
      payloadName,
    );
    const signature = await findExpectedFile(
      artifactsDirectory,
      target.artifactDirectory,
      `${payloadName}.sig`,
    );
    await copyFile(payload, join(outputDirectory, payloadName));
    await copyFile(signature, join(outputDirectory, `${payloadName}.sig`));
  }

  for (const installer of RELEASE_INSTALLERS) {
    const filename = installer.filename(version);
    const source = await findExpectedFile(
      artifactsDirectory,
      installer.artifactDirectory,
      filename,
    );
    await copyFile(source, join(outputDirectory, filename));
  }

  await createUpdaterManifest({
    artifactsDirectory,
    outputPath: join(outputDirectory, "latest.json"),
    repository,
    tag,
    version,
    pubDate,
  });

  const checksumAssets = (await readdir(outputDirectory)).sort();
  const checksumLines = await Promise.all(
    checksumAssets.map(async (filename) => {
      const digest = await sha256(join(outputDirectory, filename));
      return `${digest}  ${filename}`;
    }),
  );
  await writeFile(
    join(outputDirectory, "SHA256SUMS.txt"),
    `${checksumLines.join("\n")}\n`,
    "utf8",
  );
}

async function main() {
  const args = process.argv.slice(2);
  await assembleRelease({
    artifactsDirectory: resolve(requiredArgument(args, "--artifacts")),
    outputDirectory: resolve(requiredArgument(args, "--output")),
    repository: requiredArgument(args, "--repository"),
    tag: requiredArgument(args, "--tag"),
    version: requiredArgument(args, "--version"),
    pubDate: args.includes("--pub-date") ? requiredArgument(args, "--pub-date") : undefined,
  });
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  });
}
