import { z } from "zod";

export const mediaKindSchema = z.enum(["gif", "apng", "webp", "png", "jpeg"]);
export type MediaKind = z.infer<typeof mediaKindSchema>;

export const mediaItemSchema = z.object({
  id: z.string().uuid(),
  contentHash: z.string().regex(/^[a-f0-9]{64}$/),
  displayName: z.string(),
  managedPath: z.string(),
  mediaKind: mediaKindSchema,
  width: z.number().int().positive(),
  height: z.number().int().positive(),
  frameCount: z.number().int().positive(),
  durationMs: z.number().int().nonnegative(),
  isFavorite: z.boolean(),
  tags: z.array(z.string()),
  createdAt: z.string().datetime({ offset: true }),
  updatedAt: z.string().datetime({ offset: true }),
});
export type MediaItem = z.infer<typeof mediaItemSchema>;

export const mediaPageSchema = z.object({
  items: z.array(mediaItemSchema),
  total: z.number().int().nonnegative(),
});
export type MediaPage = z.infer<typeof mediaPageSchema>;

export const importReportSchema = z.object({
  outcomes: z.array(
    z.discriminatedUnion("status", [
      z.object({ status: z.literal("imported"), item: mediaItemSchema }),
      z.object({ status: z.literal("duplicate"), item: mediaItemSchema }),
    ]),
  ),
  failures: z.array(
    z.object({
      path: z.string(),
      code: z.string(),
      message: z.string(),
    }),
  ),
});
export type ImportReport = z.infer<typeof importReportSchema>;

export const overlaySchema = z.object({
  id: z.string().uuid(),
  mediaId: z.string().uuid(),
  monitorId: z.string().nullable(),
  x: z.number().finite(),
  y: z.number().finite(),
  width: z.number().positive(),
  opacity: z.number().min(0).max(1),
  speed: z.number().min(0.25).max(4),
  flipHorizontal: z.boolean(),
  isPlaying: z.boolean(),
  zOrder: z.number().int(),
  isActive: z.boolean(),
  createdAt: z.string().datetime({ offset: true }),
  updatedAt: z.string().datetime({ offset: true }),
});
export type OverlayInstance = z.infer<typeof overlaySchema>;

export const settingsSchema = z.object({
  launchAtLogin: z.boolean(),
  restoreOverlays: z.boolean(),
  closeToTray: z.boolean(),
  collectionView: z.enum(["grid", "list"]),
  defaultOverlayWidth: z.number().int().min(64).max(2048),
  defaultOpacity: z.number().min(0.1).max(1),
  defaultSpeed: z.number().min(0.25).max(4),
  cacheLimitMb: z.number().int().min(64).max(16_384),
});
export type AppSettings = z.infer<typeof settingsSchema>;

export const deleteReportSchema = z.object({
  mediaId: z.string().uuid(),
  overlaysRemoved: z.number().int().nonnegative(),
});
export type DeleteReport = z.infer<typeof deleteReportSchema>;

export interface LibraryQuery {
  search?: string;
  mediaKind?: MediaKind;
  favoriteOnly?: boolean;
  tag?: string;
  limit?: number;
  offset?: number;
}

export interface NewOverlay {
  mediaId: string;
  monitorId: string | null;
  x: number;
  y: number;
}

export interface OverlayPatch {
  monitorId?: string | null;
  x?: number;
  y?: number;
  width?: number;
  opacity?: number;
  speed?: number;
  flipHorizontal?: boolean;
  isPlaying?: boolean;
  zOrder?: number;
  isActive?: boolean;
}
