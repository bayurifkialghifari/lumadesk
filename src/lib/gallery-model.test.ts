import { describe, expect, it } from "vitest";
import {
  ALL_GALLERY_FORMATS,
  availableGalleryFormats,
  availableGalleryTags,
  browserGalleryCatalog,
  filterGalleryItems,
  formatGalleryBytes,
} from "./gallery-model";

const catalog = browserGalleryCatalog();

describe("community gallery model", () => {
  it("parses the bundled catalog through the native response contract", () => {
    expect(catalog.schemaVersion).toBe(1);
    expect(catalog.items).toHaveLength(4);
    expect(catalog.source).toBe("bundled");
  });

  it("searches titles, descriptions, authors, and tags", () => {
    const matches = filterGalleryItems(catalog.items, {
      search: "synthwave",
      tag: "",
      mediaFormat: ALL_GALLERY_FORMATS,
    });
    expect(matches.map((item) => item.id)).toEqual(["signal-ghost"]);
  });

  it("combines tag and format filters", () => {
    const matches = filterGalleryItems(catalog.items, {
      search: "",
      tag: "ambient",
      mediaFormat: "webp",
    });
    expect(matches.map((item) => item.id)).toEqual(["neon-orbit"]);
  });

  it("collects stable filter options", () => {
    expect(availableGalleryFormats(catalog.items)).toEqual(["apng", "gif", "webp"]);
    expect(availableGalleryTags(catalog.items)).toContain("ocean");
  });

  it("formats catalog asset sizes", () => {
    expect(formatGalleryBytes(512)).toBe("512 B");
    expect(formatGalleryBytes(11_442)).toBe("11 KB");
    expect(formatGalleryBytes(2_621_440)).toBe("2.5 MB");
  });
});
