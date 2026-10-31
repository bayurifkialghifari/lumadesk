import type { Readable } from "svelte/store";
import type { CollectionNotice, ToastTone } from "./collection-model";
import type {
  DesktopOverlay,
  EditRecipe,
  MediaItem,
  OverlayId,
  OverlayInstance,
  OverlayPatch,
} from "./contracts";

export const MIN_OVERLAY_WIDTH = 64;
export const MAX_OVERLAY_WIDTH = 2_048;

export interface DesktopState {
  readonly nativeAvailable: boolean;
  readonly ready: boolean;
  readonly loading: boolean;
  readonly refreshing: boolean;
  readonly overlays: readonly DesktopOverlay[];
  readonly assetUrls: Readonly<Record<string, string>>;
  readonly editRecipes: Readonly<Record<string, EditRecipe>>;
  readonly busyIds: ReadonlySet<OverlayId>;
  readonly batchBusy: boolean;
  readonly loadError: string | null;
  readonly notice: CollectionNotice | null;
}

export interface DesktopController extends Readable<DesktopState> {
  readonly initialize: () => Promise<void>;
  readonly reload: () => Promise<void>;
  readonly update: (id: OverlayId, patch: OverlayPatch) => Promise<void>;
  readonly duplicate: (id: OverlayId) => Promise<void>;
  readonly close: (id: OverlayId) => Promise<void>;
  readonly pauseAll: () => Promise<void>;
  readonly closeAll: () => Promise<void>;
  readonly notify: (tone: ToastTone, message: string) => void;
  readonly dismissNotice: () => void;
  readonly destroy: () => void;
}

export function overlayHeight(item: {
  readonly overlay: Pick<OverlayInstance, "width">;
  readonly media: Pick<MediaItem, "width" | "height">;
}): number {
  return Math.round((item.overlay.width * item.media.height) / item.media.width);
}

export function normalizeOverlayWidth(value: number): number {
  if (!Number.isFinite(value)) return MIN_OVERLAY_WIDTH;
  return Math.min(MAX_OVERLAY_WIDTH, Math.max(MIN_OVERLAY_WIDTH, Math.round(value)));
}

export function opacityFromPercent(value: number): number {
  if (!Number.isFinite(value)) return 1;
  return Math.min(1, Math.max(0.1, value / 100));
}

export function normalizeOverlaySpeed(value: number): number {
  if (!Number.isFinite(value)) return 1;
  return Math.min(4, Math.max(0.25, value));
}
