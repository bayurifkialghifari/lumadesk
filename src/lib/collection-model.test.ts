import { describe, expect, it } from "vitest";
import type { CollectionState } from "./collection-model";
import {
  availableTags,
  buildLibraryQuery,
  DEFAULT_SETTINGS,
  lastCollectionPage,
} from "./collection-model";
import type { MediaItem } from "./contracts";

const baseState: CollectionState = {
  nativeAvailable: true,
  ready: true,
  loading: false,
  refreshing: false,
  importing: false,
  dragActive: false,
  items: [],
  total: 0,
  page: 2,
  filters: { search: "  pixel  ", mediaFormat: "webp", favoriteOnly: true, tag: " loop " },
  settings: DEFAULT_SETTINGS,
  selectedId: null,
  assetUrls: {},
  editRecipes: {},
  busyIds: new Set(),
  loadError: null,
  notice: null,
  importReport: null,
};

describe("collection model", () => {
  it("builds a normalized paginated query", () => {
    expect(buildLibraryQuery(baseState)).toEqual({
      search: "pixel",
      mediaFormat: "webp",
      favoriteOnly: true,
      tag: "loop",
      limit: 24,
      offset: 48,
    });
  });

  it("returns unique tags in locale-aware order", () => {
    const items = [
      { tags: ["pixel", "biru"] },
      { tags: ["Pixel", "ambient"] },
    ] satisfies readonly Pick<MediaItem, "tags">[];

    expect(availableTags(items)).toEqual(["ambient", "biru", "pixel", "Pixel"]);
  });

  it("clamps pagination to an existing page", () => {
    expect(lastCollectionPage(0)).toBe(0);
    expect(lastCollectionPage(24)).toBe(0);
    expect(lastCollectionPage(25)).toBe(1);
  });
});
