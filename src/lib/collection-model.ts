import type { Readable } from "svelte/store";
import type {
  AppSettings,
  EditRecipe,
  ImportReport,
  LibraryQuery,
  MediaFormat,
  MediaId,
  MediaItem,
} from "./contracts";

export const COLLECTION_PAGE_SIZE = 24;
export const ALL_MEDIA_FORMATS = "all";
export type ToastTone = "success" | "error" | "info";

export type MediaFormatFilter = MediaFormat | typeof ALL_MEDIA_FORMATS;

export interface CollectionFilters {
  readonly search: string;
  readonly mediaFormat: MediaFormatFilter;
  readonly favoriteOnly: boolean;
  readonly tag: string;
}

export interface CollectionNotice {
  readonly id: number;
  readonly tone: ToastTone;
  readonly message: string;
}

export interface CollectionState {
  readonly nativeAvailable: boolean;
  readonly ready: boolean;
  readonly loading: boolean;
  readonly refreshing: boolean;
  readonly importing: boolean;
  readonly dragActive: boolean;
  readonly items: readonly MediaItem[];
  readonly total: number;
  readonly page: number;
  readonly filters: CollectionFilters;
  readonly settings: AppSettings;
  readonly selectedId: MediaId | null;
  readonly assetUrls: Readonly<Record<string, string>>;
  readonly editRecipes: Readonly<Record<string, EditRecipe>>;
  readonly busyIds: ReadonlySet<MediaId>;
  readonly loadError: string | null;
  readonly notice: CollectionNotice | null;
  readonly importReport: ImportReport | null;
}

export interface CollectionController extends Readable<CollectionState> {
  readonly initialize: () => Promise<void>;
  readonly reload: () => Promise<void>;
  readonly setSearch: (search: string) => void;
  readonly setMediaFormat: (mediaFormat: MediaFormatFilter) => void;
  readonly setFavoriteOnly: (favoriteOnly: boolean) => void;
  readonly setTag: (tag: string) => void;
  readonly setPage: (page: number) => void;
  readonly setView: (view: AppSettings["collectionView"]) => Promise<void>;
  readonly saveSettings: (settings: AppSettings) => Promise<boolean>;
  readonly select: (id: MediaId | null) => void;
  readonly setDragActive: (active: boolean) => void;
  readonly importPaths: (paths: readonly string[]) => Promise<void>;
  readonly toggleFavorite: (item: MediaItem) => Promise<void>;
  readonly showOverlay: (item: MediaItem) => Promise<void>;
  readonly rename: (item: MediaItem, name: string) => Promise<boolean>;
  readonly replaceTags: (item: MediaItem, tags: readonly string[]) => Promise<boolean>;
  readonly remove: (item: MediaItem) => Promise<boolean>;
  readonly notify: (tone: ToastTone, message: string) => void;
  readonly dismissNotice: () => void;
  readonly closeImportReport: () => void;
  readonly destroy: () => void;
}

export type CollectionDialog =
  | { readonly kind: "rename"; readonly item: MediaItem }
  | { readonly kind: "tags"; readonly item: MediaItem }
  | { readonly kind: "delete"; readonly item: MediaItem };

export const DEFAULT_SETTINGS = {
  theme: "dark",
  launchAtLogin: false,
  restoreOverlays: true,
  closeToTray: true,
  collectionView: "grid",
  defaultOverlayWidth: 320,
  defaultOpacity: 1,
  defaultSpeed: 1,
} satisfies AppSettings;

export function buildLibraryQuery(state: CollectionState): LibraryQuery {
  const search = state.filters.search.trim();
  const tag = state.filters.tag.trim();
  return {
    limit: COLLECTION_PAGE_SIZE,
    offset: state.page * COLLECTION_PAGE_SIZE,
    ...(search === "" ? {} : { search }),
    ...(tag === "" ? {} : { tag }),
    ...(state.filters.mediaFormat === ALL_MEDIA_FORMATS
      ? {}
      : { mediaFormat: state.filters.mediaFormat }),
    ...(state.filters.favoriteOnly ? { favoriteOnly: true } : {}),
  };
}

export function lastCollectionPage(total: number): number {
  return Math.max(0, Math.ceil(total / COLLECTION_PAGE_SIZE) - 1);
}

export function availableTags(items: readonly Pick<MediaItem, "tags">[]): readonly string[] {
  return [...new Set(items.flatMap((item) => item.tags))].sort((left, right) =>
    left.localeCompare(right, "en", { sensitivity: "base" }),
  );
}

export function hasActiveFilters(filters: CollectionFilters): boolean {
  return (
    filters.search.trim() !== "" ||
    filters.mediaFormat !== ALL_MEDIA_FORMATS ||
    filters.favoriteOnly ||
    filters.tag.trim() !== ""
  );
}
