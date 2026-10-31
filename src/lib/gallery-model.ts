import catalogData from "../../gallery/catalog.v1.json";
import moonJellyUrl from "../assets/gallery/moon-jelly.png?url";
import neonOrbitUrl from "../assets/gallery/neon-orbit.webp?url";
import pixelBloomUrl from "../assets/gallery/pixel-bloom.gif?url";
import signalGhostUrl from "../assets/gallery/signal-ghost.webp?url";
import {
  type GalleryCatalogResponse,
  type GalleryItem,
  type GalleryMediaFormat,
  galleryCatalogResponseSchema,
} from "./contracts";

export const ALL_GALLERY_FORMATS = "all" as const;
export type GalleryFormatFilter = typeof ALL_GALLERY_FORMATS | GalleryMediaFormat;

export interface GalleryFilters {
  readonly search: string;
  readonly tag: string;
  readonly mediaFormat: GalleryFormatFilter;
}

const browserPreviewUrls: Readonly<Record<string, string>> = {
  "moon-jelly": moonJellyUrl,
  "neon-orbit": neonOrbitUrl,
  "pixel-bloom": pixelBloomUrl,
  "signal-ghost": signalGhostUrl,
};

export function browserGalleryCatalog(): GalleryCatalogResponse {
  return galleryCatalogResponseSchema.parse({
    ...catalogData,
    source: "bundled",
    installedHashes: [],
  });
}

export function browserGalleryPreviewUrls(): Readonly<Record<string, string>> {
  return browserPreviewUrls;
}

export function filterGalleryItems(
  items: readonly GalleryItem[],
  filters: GalleryFilters,
): GalleryItem[] {
  const query = filters.search.trim().toLocaleLowerCase();
  const tag = filters.tag.toLocaleLowerCase();
  return items.filter((item) => {
    if (filters.mediaFormat !== ALL_GALLERY_FORMATS) {
      if (item.download.mediaFormat !== filters.mediaFormat) return false;
    }
    if (tag !== "" && !item.tags.some((candidate) => candidate.toLocaleLowerCase() === tag)) {
      return false;
    }
    if (query === "") return true;
    return [item.title, item.description, item.author, ...item.tags]
      .join(" ")
      .toLocaleLowerCase()
      .includes(query);
  });
}

export function availableGalleryTags(items: readonly GalleryItem[]): string[] {
  return [...new Set(items.flatMap((item) => item.tags))].sort((left, right) =>
    left.localeCompare(right),
  );
}

export function availableGalleryFormats(items: readonly GalleryItem[]): GalleryMediaFormat[] {
  return [...new Set(items.map((item) => item.download.mediaFormat))].sort();
}

export function formatGalleryBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(bytes < 10 * 1024 * 1024 ? 1 : 0)} MB`;
}
