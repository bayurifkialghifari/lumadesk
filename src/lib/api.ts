import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { z } from "zod";
import {
  type AiMaskBatch,
  type AiModel,
  type AiModelStatus,
  type AppSettings,
  aiMaskBatchSchema,
  aiModelStatusSchema,
  type CommandErrorPayload,
  type CreatorChanges,
  type CreatorChangesInput,
  commandErrorSchema,
  creatorChangesSchema,
  type DeleteReport,
  type DesktopOverlay,
  type DesktopScene,
  type DesktopSceneId,
  type DesktopSceneSummary,
  deleteReportSchema,
  desktopOverlaySchema,
  desktopSceneSchema,
  desktopSceneSummarySchema,
  type EditRecipe,
  type EditRecipePatch,
  editRecipeSchema,
  type FrameBatch,
  frameBatchSchema,
  type ImportReport,
  importReportSchema,
  type LibraryQuery,
  type MediaId,
  type MediaItem,
  type MediaPage,
  type MediaVariant,
  type MonitorDescriptor,
  mediaItemSchema,
  mediaPageSchema,
  mediaVariantSchema,
  monitorDescriptorSchema,
  type NewOverlay,
  type OverlayBatchPatch,
  type OverlayId,
  type OverlayInstance,
  type OverlayPatch,
  overlaySchema,
  type SpriteSequence,
  type SpriteSequenceInput,
  settingsSchema,
  spriteSequenceSchema,
} from "./contracts";

export class NativeCommandError extends Error {
  public readonly code: CommandErrorPayload["code"];

  public constructor(payload: CommandErrorPayload, options?: ErrorOptions) {
    super(payload.message, options);
    this.name = "NativeCommandError";
    this.code = payload.code;
  }
}

async function invokeParsed<TSchema extends z.ZodType>(
  command: string,
  schema: TSchema,
  args?: Record<string, unknown>,
): Promise<z.output<TSchema>> {
  let payload: unknown;
  try {
    payload = await invoke<unknown>(command, args);
  } catch (error) {
    const parsed = commandErrorSchema.safeParse(error);
    if (parsed.success) {
      throw new NativeCommandError(parsed.data, { cause: error });
    }
    if (error instanceof Error) {
      throw error;
    }
    throw new NativeCommandError(
      { code: "invalid_data", message: "The application response could not be read." },
      { cause: error },
    );
  }
  const parsed = schema.safeParse(payload);
  if (!parsed.success) {
    throw new NativeCommandError(
      { code: "invalid_data", message: "The application response could not be read." },
      { cause: parsed.error },
    );
  }
  return parsed.data;
}

export function listMedia(query: LibraryQuery = {}): Promise<MediaPage> {
  return invokeParsed("list_media", mediaPageSchema, { query });
}

export function importMedia(paths: readonly string[]): Promise<ImportReport> {
  return invokeParsed("import_media", importReportSchema, { paths });
}

export function renameMedia(id: MediaId, name: string): Promise<MediaItem> {
  return invokeParsed("rename_media", mediaItemSchema, { id, name });
}

export function setFavorite(id: MediaId, favorite: boolean): Promise<MediaItem> {
  return invokeParsed("set_favorite", mediaItemSchema, { id, favorite });
}

export function replaceTags(id: MediaId, tags: readonly string[]): Promise<MediaItem> {
  return invokeParsed("replace_tags", mediaItemSchema, { id, tags });
}

export function deleteMedia(id: MediaId): Promise<DeleteReport> {
  return invokeParsed("delete_media", deleteReportSchema, { id });
}

export async function mediaAssetUrl(id: MediaId): Promise<string> {
  const path = await invokeParsed("resolve_media_path", z.string(), { id });
  return convertFileSrc(path);
}

export async function getFrameBatch(
  id: MediaId,
  offset: number,
  limit: number,
): Promise<FrameBatch> {
  const batch = await invokeParsed("get_frame_batch", frameBatchSchema, { id, offset, limit });
  return {
    ...batch,
    frames: batch.frames.map((frame) => ({ ...frame, path: convertFileSrc(frame.path) })),
  };
}

export function getEditRecipe(id: MediaId): Promise<EditRecipe> {
  return invokeParsed("get_edit_recipe", editRecipeSchema, { id });
}

export function updateEditRecipe(id: MediaId, patch: EditRecipePatch): Promise<EditRecipe> {
  return invokeParsed("update_edit_recipe", editRecipeSchema, { id, patch });
}

export function saveCreatorChanges(input: CreatorChangesInput): Promise<CreatorChanges> {
  return invokeParsed("save_creator_changes", creatorChangesSchema, { input });
}

export function getSpriteSequence(id: MediaId): Promise<SpriteSequence | null> {
  return invokeParsed("get_sprite_sequence", spriteSequenceSchema.nullable(), { id });
}

export function setSpriteSequence(input: SpriteSequenceInput): Promise<SpriteSequence> {
  return invokeParsed("set_sprite_sequence", spriteSequenceSchema, { input });
}

export async function clearSpriteSequence(id: MediaId): Promise<void> {
  await invokeParsed("clear_sprite_sequence", z.null(), { id });
}

export function listMediaVariants(id: MediaId): Promise<MediaVariant[]> {
  return invokeParsed("list_media_variants", z.array(mediaVariantSchema), { id });
}

export function listOverlays(): Promise<OverlayInstance[]> {
  return invokeParsed("list_overlays", z.array(overlaySchema));
}

export function listDesktopOverlays(): Promise<DesktopOverlay[]> {
  return invokeParsed("list_desktop_overlays", z.array(desktopOverlaySchema));
}

export function createOverlay(input: NewOverlay): Promise<OverlayInstance> {
  return invokeParsed("create_overlay", overlaySchema, { input });
}

export function showMediaOverlay(id: MediaId): Promise<OverlayInstance> {
  return invokeParsed("show_media_overlay", overlaySchema, { id });
}

export function updateOverlay(id: OverlayId, patch: OverlayPatch): Promise<OverlayInstance> {
  return invokeParsed("update_overlay", overlaySchema, { id, patch });
}

export function moveOverlay(
  id: OverlayId,
  monitorId: string | null,
  x: number,
  y: number,
): Promise<OverlayInstance> {
  return invokeParsed("move_overlay", overlaySchema, { id, monitorId, x, y });
}

export function moveOverlayToMonitor(id: OverlayId, monitorName: string): Promise<OverlayInstance> {
  return invokeParsed("move_overlay_to_monitor", overlaySchema, { id, monitorName });
}

export function duplicateOverlay(id: OverlayId): Promise<OverlayInstance> {
  return invokeParsed("duplicate_overlay", overlaySchema, { id });
}

export async function removeOverlay(id: OverlayId): Promise<void> {
  await invokeParsed("remove_overlay", z.null(), { id });
}

export function listMonitors(): Promise<MonitorDescriptor[]> {
  return invokeParsed("list_monitors", z.array(monitorDescriptorSchema));
}

export function listDesktopScenes(): Promise<DesktopSceneSummary[]> {
  return invokeParsed("list_desktop_scenes", z.array(desktopSceneSummarySchema));
}

export function createDesktopScene(name: string): Promise<DesktopScene> {
  return invokeParsed("create_desktop_scene", desktopSceneSchema, { name });
}

export function updateDesktopScene(id: DesktopSceneId): Promise<DesktopScene> {
  return invokeParsed("update_desktop_scene", desktopSceneSchema, { id });
}

export function renameDesktopScene(id: DesktopSceneId, name: string): Promise<DesktopScene> {
  return invokeParsed("rename_desktop_scene", desktopSceneSchema, { id, name });
}

export function duplicateDesktopScene(id: DesktopSceneId): Promise<DesktopScene> {
  return invokeParsed("duplicate_desktop_scene", desktopSceneSchema, { id });
}

export async function deleteDesktopScene(id: DesktopSceneId): Promise<void> {
  await invokeParsed("delete_desktop_scene", z.null(), { id });
}

export function applyDesktopScene(id: DesktopSceneId): Promise<OverlayInstance[]> {
  return invokeParsed("apply_desktop_scene", z.array(overlaySchema), { id });
}

export function patchAllOverlays(patch: OverlayBatchPatch): Promise<OverlayInstance[]> {
  return invokeParsed("patch_all_overlays", z.array(overlaySchema), { patch });
}

export function getSettings(): Promise<AppSettings> {
  return invokeParsed("get_settings", settingsSchema);
}

export function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return invokeParsed("update_settings", settingsSchema, { settings });
}

export async function openDataFolder(): Promise<void> {
  await invokeParsed("open_data_folder", z.null());
}

export async function clearGeneratedCache(): Promise<void> {
  await invokeParsed("clear_generated_cache", z.null());
}

export function listAiModels(): Promise<AiModelStatus[]> {
  return invokeParsed("list_ai_models", z.array(aiModelStatusSchema));
}

export function downloadAiModel(model: AiModel): Promise<AiModelStatus> {
  return invokeParsed("download_ai_model", aiModelStatusSchema, { model });
}

export function removeAiModel(model: AiModel): Promise<AiModelStatus> {
  return invokeParsed("remove_ai_model", aiModelStatusSchema, { model });
}

export async function getAiMaskBatch(
  id: MediaId,
  model: AiModel,
  offset: number,
  limit: number,
): Promise<AiMaskBatch> {
  const batch = await invokeParsed("get_ai_mask_batch", aiMaskBatchSchema, {
    id,
    model,
    offset,
    limit,
  });
  return {
    ...batch,
    masks: batch.masks.map((mask) => ({ ...mask, path: convertFileSrc(mask.path) })),
  };
}
