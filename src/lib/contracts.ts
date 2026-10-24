import { z } from "zod";

export const errorCodeSchema = z.enum([
  "io",
  "database",
  "unsupported_media",
  "invalid_media",
  "invalid_input",
  "invalid_data",
  "not_found",
  "schema_too_new",
]);
export type ErrorCode = z.infer<typeof errorCodeSchema>;

export const commandErrorSchema = z.object({
  code: errorCodeSchema,
  message: z.string().min(1),
});
export type CommandErrorPayload = z.infer<typeof commandErrorSchema>;

export const mediaIdSchema = z.string().uuid().brand<"MediaId">();
export type MediaId = z.infer<typeof mediaIdSchema>;

export const overlayIdSchema = z.string().uuid().brand<"OverlayId">();
export type OverlayId = z.infer<typeof overlayIdSchema>;

export const mediaKindSchema = z.enum(["gif", "apng", "webp", "png", "jpeg"]);
export type MediaKind = z.infer<typeof mediaKindSchema>;

export const mediaItemSchema = z.object({
  id: mediaIdSchema,
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
      code: errorCodeSchema,
      message: z.string(),
    }),
  ),
});
export type ImportReport = z.infer<typeof importReportSchema>;

export const overlaySchema = z.object({
  id: overlayIdSchema,
  mediaId: mediaIdSchema,
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
  mediaId: mediaIdSchema,
  overlaysRemoved: z.number().int().nonnegative(),
});
export type DeleteReport = z.infer<typeof deleteReportSchema>;

export const frameAssetSchema = z.object({
  index: z.number().int().nonnegative(),
  durationMs: z.number().int().nonnegative(),
  path: z.string().min(1),
});
export type FrameAsset = z.infer<typeof frameAssetSchema>;

export const frameBatchSchema = z.object({
  mediaId: mediaIdSchema,
  width: z.number().int().positive(),
  height: z.number().int().positive(),
  totalFrames: z.number().int().positive(),
  totalDurationMs: z.number().int().nonnegative(),
  frames: z.array(frameAssetSchema).max(64),
});
export type FrameBatch = z.infer<typeof frameBatchSchema>;

export interface LibraryQuery {
  readonly search?: string;
  readonly mediaKind?: MediaKind;
  readonly favoriteOnly?: boolean;
  readonly tag?: string;
  readonly limit?: number;
  readonly offset?: number;
}

export interface NewOverlay {
  readonly mediaId: MediaId;
  readonly monitorId: string | null;
  readonly x: number;
  readonly y: number;
}

export interface OverlayPatch {
  readonly monitorId?: string | null;
  readonly x?: number;
  readonly y?: number;
  readonly width?: number;
  readonly opacity?: number;
  readonly speed?: number;
  readonly flipHorizontal?: boolean;
  readonly isPlaying?: boolean;
  readonly zOrder?: number;
  readonly isActive?: boolean;
}
