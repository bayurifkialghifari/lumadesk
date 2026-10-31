import { mkdir, readFile, writeFile } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { RELEASE_TARGETS } from "./release-targets.mjs";
import {
  listFiles,
  requiredArgument,
  validateReleaseIdentity,
} from "./release-utils.mjs";

export async function createUpdaterManifest({
  artifactsDirectory,
  outputPath,
  repository,
  tag,
  version,
  pubDate = new Date().toISOString(),
}) {
  validateReleaseIdentity({ repository, tag, version });

  const platforms = {};
  const payloadNames = new Set();

  for (const target of RELEASE_TARGETS) {
    const targetDirectory = join(artifactsDirectory, target.artifactDirectory);
    const files = await listFiles(targetDirectory);
    const expectedName = target.payloadName(version);
    const payloads = files.filter((file) => basename(file) === expectedName);
    if (payloads.length !== 1) {
      throw new Error(
        `Expected exactly one updater payload ${expectedName} in ${target.artifactDirectory}, found ${payloads.length}.`,
      );
    }

    const payload = payloads[0];
    const signaturePath = `${payload}.sig`;
    if (!files.includes(signaturePath)) {
      throw new Error(
        `Missing updater signature ${basename(signaturePath)} in ${target.artifactDirectory}.`,
      );
    }

    const payloadName = basename(payload);
    if (payloadNames.has(payloadName)) {
      throw new Error(`Duplicate updater payload filename ${payloadName}.`);
    }
    payloadNames.add(payloadName);

    const signature = (await readFile(signaturePath, "utf8")).trim();
    if (signature.length === 0) {
      throw new Error(`Updater signature ${basename(signaturePath)} is empty.`);
    }

    platforms[target.platform] = {
      signature,
      url: `https://github.com/${repository}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(payloadName)}`,
    };
  }

  const manifest = {
    version,
    notes: `See the LumaDesk ${tag} release notes on GitHub.`,
    pub_date: pubDate,
    platforms,
  };
  await mkdir(dirname(outputPath), { recursive: true });
  await writeFile(outputPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
}

async function main() {
  const args = process.argv.slice(2);
  await createUpdaterManifest({
    artifactsDirectory: resolve(requiredArgument(args, "--artifacts")),
    outputPath: resolve(requiredArgument(args, "--output")),
    repository: requiredArgument(args, "--repository"),
    tag: requiredArgument(args, "--tag"),
    version: requiredArgument(args, "--version"),
    pubDate:
      args.includes("--pub-date") ? requiredArgument(args, "--pub-date") : new Date().toISOString(),
  });
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  });
}
