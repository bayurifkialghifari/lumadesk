import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { z } from "zod";
import {
  type AppSettings,
  type CommandErrorPayload,
  commandErrorSchema,
  type DeleteReport,
  deleteReportSchema,
  type ImportReport,
  importReportSchema,
  type LibraryQuery,
  type MediaId,
  type MediaItem,
  type MediaPage,
  mediaItemSchema,
  mediaPageSchema,
  type NewOverlay,
  type OverlayId,
  type OverlayInstance,
  type OverlayPatch,
  overlaySchema,
  settingsSchema,
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
      { code: "invalid_data", message: "Respons aplikasi tidak dapat dibaca." },
      { cause: error },
    );
  }
  const parsed = schema.safeParse(payload);
  if (!parsed.success) {
    throw new NativeCommandError(
      { code: "invalid_data", message: "Respons aplikasi tidak dapat dibaca." },
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

export function listOverlays(): Promise<OverlayInstance[]> {
  return invokeParsed("list_overlays", z.array(overlaySchema));
}

export function createOverlay(input: NewOverlay): Promise<OverlayInstance> {
  return invokeParsed("create_overlay", overlaySchema, { input });
}

export function updateOverlay(id: OverlayId, patch: OverlayPatch): Promise<OverlayInstance> {
  return invokeParsed("update_overlay", overlaySchema, { id, patch });
}

export async function removeOverlay(id: OverlayId): Promise<void> {
  await invokeParsed("remove_overlay", z.null(), { id });
}

export function getSettings(): Promise<AppSettings> {
  return invokeParsed("get_settings", settingsSchema);
}

export function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return invokeParsed("update_settings", settingsSchema, { settings });
}
