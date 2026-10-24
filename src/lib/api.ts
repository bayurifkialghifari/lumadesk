import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { z } from "zod";
import {
  type AppSettings,
  type DeleteReport,
  deleteReportSchema,
  type ImportReport,
  importReportSchema,
  type LibraryQuery,
  type MediaItem,
  type MediaPage,
  mediaItemSchema,
  mediaPageSchema,
  type NewOverlay,
  type OverlayInstance,
  type OverlayPatch,
  overlaySchema,
  settingsSchema,
} from "./contracts";

async function invokeParsed<TSchema extends z.ZodType>(
  command: string,
  schema: TSchema,
  args?: Record<string, unknown>,
): Promise<z.output<TSchema>> {
  const payload = await invoke<unknown>(command, args);
  return schema.parse(payload);
}

export function listMedia(query: LibraryQuery = {}): Promise<MediaPage> {
  return invokeParsed("list_media", mediaPageSchema, { query });
}

export function importMedia(paths: string[]): Promise<ImportReport> {
  return invokeParsed("import_media", importReportSchema, { paths });
}

export function renameMedia(id: string, name: string): Promise<MediaItem> {
  return invokeParsed("rename_media", mediaItemSchema, { id, name });
}

export function setFavorite(id: string, favorite: boolean): Promise<MediaItem> {
  return invokeParsed("set_favorite", mediaItemSchema, { id, favorite });
}

export function replaceTags(id: string, tags: string[]): Promise<MediaItem> {
  return invokeParsed("replace_tags", mediaItemSchema, { id, tags });
}

export function deleteMedia(id: string): Promise<DeleteReport> {
  return invokeParsed("delete_media", deleteReportSchema, { id });
}

export async function mediaAssetUrl(id: string): Promise<string> {
  const path = await invokeParsed("resolve_media_path", z.string(), { id });
  return convertFileSrc(path);
}

export function listOverlays(): Promise<OverlayInstance[]> {
  return invokeParsed("list_overlays", z.array(overlaySchema));
}

export function createOverlay(input: NewOverlay): Promise<OverlayInstance> {
  return invokeParsed("create_overlay", overlaySchema, { input });
}

export function updateOverlay(id: string, patch: OverlayPatch): Promise<OverlayInstance> {
  return invokeParsed("update_overlay", overlaySchema, { id, patch });
}

export async function removeOverlay(id: string): Promise<void> {
  await invokeParsed("remove_overlay", z.null(), { id });
}

export function getSettings(): Promise<AppSettings> {
  return invokeParsed("get_settings", settingsSchema);
}

export function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return invokeParsed("update_settings", settingsSchema, { settings });
}
