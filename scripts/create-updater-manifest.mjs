import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const targets = [
  { directory: "linux-x86_64", platform: "linux-x86_64-appimage", suffix: ".AppImage" },
  { directory: "linux-x86_64", platform: "linux-x86_64-deb", suffix: ".deb" },
  { directory: "linux-x86_64", platform: "linux-x86_64-rpm", suffix: ".rpm" },
  { directory: "linux-aarch64", platform: "linux-aarch64-appimage", suffix: ".AppImage" },
  { directory: "linux-aarch64", platform: "linux-aarch64-deb", suffix: ".deb" },
  { directory: "linux-aarch64", platform: "linux-aarch64-rpm", suffix: ".rpm" },
  { directory: "macos-aarch64", platform: "darwin-aarch64-app", suffix: ".app.tar.gz" },
  { directory: "macos-x86_64", platform: "darwin-x86_64-app", suffix: ".app.tar.gz" },
  { directory: "windows-x86_64", platform: "windows-x86_64-nsis", suffix: ".exe" },
  { directory: "windows-aarch64", platform: "windows-aarch64-nsis", suffix: ".exe" },
];

async function listFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(
    entries.map((entry) => {
      const path = join(directory, entry.name);
      return entry.isDirectory() ? listFiles(path) : [path];
    }),
  );
  return nested.flat();
}

function requiredArgument(args, name) {
  const index = args.indexOf(name);
  const value = index === -1 ? undefined : args[index + 1];
  if (value === undefined || value.startsWith("--")) {
    throw new Error(`Missing required argument ${name}.`);
  }
  return value;
}

export async function createUpdaterManifest({
  artifactsDirectory,
  outputPath,
  repository,
  tag,
  version,
  pubDate = new Date().toISOString(),
}) {
  if (!/^[^/\s]+\/[^/\s]+$/.test(repository)) {
    throw new Error(`Invalid GitHub repository ${repository}.`);
  }
  if (tag !== `v${version}`) {
    throw new Error(`Tag ${tag} does not match version ${version}.`);
  }

  const platforms = {};
  const payloadNames = new Set();

  for (const target of targets) {
    const targetDirectory = join(artifactsDirectory, target.directory);
    const files = await listFiles(targetDirectory);
    const payloads = files.filter((file) => file.endsWith(target.suffix));
    if (payloads.length !== 1) {
      throw new Error(
        `Expected exactly one ${target.suffix} updater payload in ${target.directory}, found ${payloads.length}.`,
      );
    }

    const payload = payloads[0];
    const signaturePath = `${payload}.sig`;
    if (!files.includes(signaturePath)) {
      throw new Error(`Missing updater signature ${basename(signaturePath)} in ${target.directory}.`);
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
