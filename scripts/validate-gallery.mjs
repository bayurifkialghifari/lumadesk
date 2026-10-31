import { createHash } from "node:crypto";
import { readFile, stat } from "node:fs/promises";
import { basename, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const catalogPath = resolve(root, "gallery/catalog.v1.json");
const assetRoot = resolve(root, "src/assets/gallery");
const supportedFormats = new Set(["gif", "apng", "webp", "png", "jpeg"]);
const expectedKeys = {
  catalog: ["schemaVersion", "revision", "items"],
  item: [
    "id",
    "title",
    "description",
    "author",
    "licenseName",
    "licenseUrl",
    "sourceUrl",
    "tags",
    "preview",
    "download",
  ],
  asset: ["url", "sha256", "sizeBytes", "mediaFormat"],
};

function fail(message) {
  throw new Error(`Gallery validation failed: ${message}`);
}

function assert(condition, message) {
  if (!condition) fail(message);
}

function assertExactKeys(value, keys, label) {
  assert(value !== null && typeof value === "object" && !Array.isArray(value), `${label} must be an object`);
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  assert(JSON.stringify(actual) === JSON.stringify(expected), `${label} has missing or unknown fields`);
}

function assertText(value, minimum, maximum, label) {
  assert(typeof value === "string", `${label} must be a string`);
  const length = [...value.trim()].length;
  assert(length >= minimum && length <= maximum, `${label} must contain ${minimum}-${maximum} characters`);
}

function httpsUrl(value, label) {
  assertText(value, 1, 2_048, label);
  let url;
  try {
    url = new URL(value);
  } catch {
    fail(`${label} is not a valid URL`);
  }
  assert(url.protocol === "https:", `${label} must use HTTPS`);
  assert(url.username === "" && url.password === "", `${label} must not contain credentials`);
  return url;
}

async function validateAsset(asset, itemId, role) {
  assertExactKeys(asset, expectedKeys.asset, `${itemId}.${role}`);
  const url = httpsUrl(asset.url, `${itemId}.${role}.url`);
  assert(supportedFormats.has(asset.mediaFormat), `${itemId}.${role} uses an unsupported format`);
  assert(Number.isSafeInteger(asset.sizeBytes) && asset.sizeBytes > 0, `${itemId}.${role}.sizeBytes must be positive`);
  assert(asset.sizeBytes <= 100 * 1024 * 1024, `${itemId}.${role} exceeds 100 MiB`);
  assert(/^[a-f0-9]{64}$/.test(asset.sha256), `${itemId}.${role}.sha256 must be lowercase SHA-256`);
  assert(url.hostname === "raw.githubusercontent.com", `${itemId}.${role} must use the repository asset host`);
  const filename = basename(decodeURIComponent(url.pathname));
  assert(filename !== "", `${itemId}.${role} URL has no filename`);
  const path = resolve(assetRoot, filename);
  assert(path.startsWith(`${assetRoot}/`), `${itemId}.${role} resolves outside the Gallery asset directory`);
  const metadata = await stat(path).catch(() => null);
  assert(metadata?.isFile() === true, `${itemId}.${role} local asset is missing: ${filename}`);
  assert(metadata.size === asset.sizeBytes, `${itemId}.${role} byte size does not match ${filename}`);
  const bytes = await readFile(path);
  const hash = createHash("sha256").update(bytes).digest("hex");
  assert(hash === asset.sha256, `${itemId}.${role} SHA-256 does not match ${filename}`);
}

async function main() {
  const raw = await readFile(catalogPath, "utf8");
  const catalog = JSON.parse(raw);
  assertExactKeys(catalog, expectedKeys.catalog, "catalog");
  assert(catalog.schemaVersion === 1, "schemaVersion must be 1");
  assert(Number.isSafeInteger(catalog.revision) && catalog.revision > 0, "revision must be positive");
  assert(Array.isArray(catalog.items) && catalog.items.length > 0 && catalog.items.length <= 500, "items must contain 1-500 entries");

  const ids = new Set();
  for (const item of catalog.items) {
    assertExactKeys(item, expectedKeys.item, "Gallery item");
    assert(typeof item.id === "string" && /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(item.id), "item IDs must use stable lowercase kebab-case");
    assert(!ids.has(item.id), `duplicate item ID: ${item.id}`);
    ids.add(item.id);
    assertText(item.title, 1, 120, `${item.id}.title`);
    assertText(item.description, 1, 500, `${item.id}.description`);
    assertText(item.author, 1, 120, `${item.id}.author`);
    assertText(item.licenseName, 1, 80, `${item.id}.licenseName`);
    httpsUrl(item.licenseUrl, `${item.id}.licenseUrl`);
    httpsUrl(item.sourceUrl, `${item.id}.sourceUrl`);
    assert(Array.isArray(item.tags) && item.tags.length > 0 && item.tags.length <= 20, `${item.id}.tags must contain 1-20 values`);
    const tags = new Set();
    for (const tag of item.tags) {
      assertText(tag, 1, 32, `${item.id}.tag`);
      const normalized = tag.toLocaleLowerCase();
      assert(!tags.has(normalized), `${item.id} contains a duplicate tag: ${tag}`);
      tags.add(normalized);
    }
    await validateAsset(item.preview, item.id, "preview");
    await validateAsset(item.download, item.id, "download");
  }

  console.log(`Validated ${catalog.items.length} Gallery items at revision ${catalog.revision}.`);
}

await main();
