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

export const desktopSceneIdSchema = z.string().uuid().brand<"DesktopSceneId">();
export type DesktopSceneId = z.infer<typeof desktopSceneIdSchema>;

export const variantIdSchema = z.string().uuid().brand<"VariantId">();
export type VariantId = z.infer<typeof variantIdSchema>;

export const assetKindSchema = z.enum(["still", "animation", "sprite_sheet", "video", "audio"]);
export type AssetKind = z.infer<typeof assetKindSchema>;

export const mediaFormatSchema = z.enum([
  "gif",
  "apng",
  "webp",
  "png",
  "jpeg",
  "mp4",
  "webm",
  "mp3",
  "wav",
  "ogg",
]);
export type MediaFormat = z.infer<typeof mediaFormatSchema>;

export const mediaItemSchema = z.object({
  id: mediaIdSchema,
  contentHash: z.string().regex(/^[a-f0-9]{64}$/),
  displayName: z.string(),
  managedPath: z.string(),
  assetKind: assetKindSchema,
  mediaFormat: mediaFormatSchema,
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

export const backgroundRemovalModeSchema = z.enum(["none", "edge", "ai_fast", "ai_quality"]);
export type BackgroundRemovalMode = z.infer<typeof backgroundRemovalModeSchema>;

export const cropRectSchema = z.object({
  x: z.number().finite().nonnegative(),
  y: z.number().finite().nonnegative(),
  width: z.number().finite().positive(),
  height: z.number().finite().positive(),
});
export type CropRect = z.infer<typeof cropRectSchema>;

export const editRecipeSchema = z.object({
  mediaId: mediaIdSchema,
  crop: cropRectSchema.nullable(),
  rotationDegrees: z.union([z.literal(0), z.literal(90), z.literal(180), z.literal(270)]),
  flipHorizontal: z.boolean(),
  backgroundRemoval: backgroundRemovalModeSchema,
  brightness: z.number().min(0).max(2),
  contrast: z.number().min(0).max(2),
  saturation: z.number().min(0).max(2),
  updatedAt: z.string().datetime({ offset: true }),
});
export type EditRecipe = z.infer<typeof editRecipeSchema>;

export interface EditRecipePatch {
  readonly crop?: CropRect | null;
  readonly rotationDegrees?: 0 | 90 | 180 | 270;
  readonly flipHorizontal?: boolean;
  readonly backgroundRemoval?: BackgroundRemovalMode;
  readonly brightness?: number;
  readonly contrast?: number;
  readonly saturation?: number;
}

export const spritePlaybackModeSchema = z.enum(["loop", "once", "ping_pong"]);
export type SpritePlaybackMode = z.infer<typeof spritePlaybackModeSchema>;

export const spriteSequenceSchema = z.object({
  mediaId: mediaIdSchema,
  columns: z.number().int().positive(),
  rows: z.number().int().positive(),
  frameCount: z.number().int().positive(),
  frameDurationMs: z.number().int().positive(),
  playbackMode: spritePlaybackModeSchema,
  updatedAt: z.string().datetime({ offset: true }),
});
export type SpriteSequence = z.infer<typeof spriteSequenceSchema>;
export type SpriteSequenceInput = Omit<SpriteSequence, "updatedAt">;

export const creatorChangesSchema = z.object({
  recipe: editRecipeSchema,
  spriteSequence: spriteSequenceSchema.nullable(),
});
export type CreatorChanges = z.infer<typeof creatorChangesSchema>;

export interface CreatorChangesInput {
  readonly mediaId: MediaId;
  readonly recipe: EditRecipePatch;
  readonly spriteSequence: SpriteSequenceInput | null;
  readonly clearSpriteSequence: boolean;
}

export const mediaVariantSchema = z.object({
  id: variantIdSchema,
  mediaId: mediaIdSchema,
  kind: z.enum(["preview", "export", "proxy", "segmentation_mask"]),
  relativePath: z.string().min(1),
  width: z.number().int().positive(),
  height: z.number().int().positive(),
  createdAt: z.string().datetime({ offset: true }),
});
export type MediaVariant = z.infer<typeof mediaVariantSchema>;

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
  removeBackground: z.boolean(),
  clickThrough: z.boolean(),
  isPlaying: z.boolean(),
  zOrder: z.number().int(),
  isActive: z.boolean(),
  createdAt: z.string().datetime({ offset: true }),
  updatedAt: z.string().datetime({ offset: true }),
});
export type OverlayInstance = z.infer<typeof overlaySchema>;

export const desktopOverlaySchema = z.object({
  overlay: overlaySchema,
  media: mediaItemSchema,
});
export type DesktopOverlay = z.infer<typeof desktopOverlaySchema>;

export const desktopSceneItemSchema = z.object({
  mediaId: mediaIdSchema,
  monitorName: z.string().min(1).nullable(),
  monitorWidth: z.number().finite().positive(),
  monitorHeight: z.number().finite().positive(),
  normalizedX: z.number().finite(),
  normalizedY: z.number().finite(),
  width: z.number().finite().positive(),
  opacity: z.number().min(0).max(1),
  speed: z.number().min(0.25).max(4),
  flipHorizontal: z.boolean(),
  removeBackground: z.boolean(),
  isPlaying: z.boolean(),
  clickThrough: z.boolean(),
  zOrder: z.number().int(),
  isActive: z.boolean(),
});
export type DesktopSceneItem = z.infer<typeof desktopSceneItemSchema>;

export const desktopSceneSchema = z.object({
  id: desktopSceneIdSchema,
  name: z.string().min(1).max(80),
  items: z.array(desktopSceneItemSchema),
  createdAt: z.string().datetime({ offset: true }),
  updatedAt: z.string().datetime({ offset: true }),
});
export type DesktopScene = z.infer<typeof desktopSceneSchema>;

export const desktopSceneSummarySchema = z.object({
  id: desktopSceneIdSchema,
  name: z.string().min(1).max(80),
  itemCount: z.number().int().nonnegative(),
  createdAt: z.string().datetime({ offset: true }),
  updatedAt: z.string().datetime({ offset: true }),
});
export type DesktopSceneSummary = z.infer<typeof desktopSceneSummarySchema>;

export const monitorDescriptorSchema = z.object({
  name: z.string().min(1),
  x: z.number().finite(),
  y: z.number().finite(),
  width: z.number().finite().positive(),
  height: z.number().finite().positive(),
  scaleFactor: z.number().finite().positive(),
  isPrimary: z.boolean(),
});
export type MonitorDescriptor = z.infer<typeof monitorDescriptorSchema>;

export const settingsSchema = z.object({
  theme: z.enum(["dark", "system"]),
  launchAtLogin: z.boolean(),
  restoreOverlays: z.boolean(),
  closeToTray: z.boolean(),
  collectionView: z.enum(["grid", "list"]),
  defaultOverlayWidth: z.number().int().min(64).max(2048),
  defaultOpacity: z.number().min(0.1).max(1),
  defaultSpeed: z.number().min(0.25).max(4),
});
export type AppSettings = z.infer<typeof settingsSchema>;

export const aiModelSchema = z.enum(["fast", "quality"]);
export type AiModel = z.infer<typeof aiModelSchema>;

export const aiModelStatusSchema = z.object({
  model: aiModelSchema,
  status: z.enum(["missing", "installed", "downloading", "removing", "error"]),
  sizeBytes: z.number().int().nonnegative(),
  installedBytes: z.number().int().nonnegative(),
  message: z.string().min(1).nullable(),
});
export type AiModelStatus = z.infer<typeof aiModelStatusSchema>;

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

export const aiMaskBatchSchema = z.object({
  mediaId: mediaIdSchema,
  model: aiModelSchema,
  masks: z.array(
    z.object({
      index: z.number().int().nonnegative(),
      path: z.string().min(1),
    }),
  ),
});
export type AiMaskBatch = z.infer<typeof aiMaskBatchSchema>;

export interface LibraryQuery {
  readonly search?: string;
  readonly assetKind?: AssetKind;
  readonly mediaFormat?: MediaFormat;
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
  readonly removeBackground?: boolean;
  readonly clickThrough?: boolean;
  readonly isPlaying?: boolean;
  readonly zOrder?: number;
  readonly isActive?: boolean;
}

export interface OverlayBatchPatch {
  readonly isPlaying?: boolean;
  readonly isActive?: boolean;
  readonly clickThrough?: boolean;
}
