import { createHash } from "node:crypto";
import { readFile, readdir } from "node:fs/promises";
import { join } from "node:path";

export function requiredArgument(args, name) {
  const index = args.indexOf(name);
  const value = index === -1 ? undefined : args[index + 1];
  if (value === undefined || value.startsWith("--")) {
    throw new Error(`Missing required argument ${name}.`);
  }
  return value;
}

export async function listFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(
    entries.map((entry) => {
      const path = join(directory, entry.name);
      return entry.isDirectory() ? listFiles(path) : [path];
    }),
  );
  return nested.flat();
}

export async function sha256(path) {
  return createHash("sha256").update(await readFile(path)).digest("hex");
}

export function validateReleaseIdentity({ repository, tag, version }) {
  if (!/^[^/\s]+\/[^/\s]+$/.test(repository)) {
    throw new Error(`Invalid GitHub repository ${repository}.`);
  }
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    throw new Error(`Version ${version} must use the MAJOR.MINOR.PATCH format.`);
  }
  if (tag !== `v${version}`) {
    throw new Error(`Tag ${tag} does not match version ${version}.`);
  }
}
