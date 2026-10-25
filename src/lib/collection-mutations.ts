import {
  deleteMedia,
  importMedia,
  renameMedia,
  replaceTags,
  setFavorite,
  showMediaOverlay,
} from "./api";
import type { CollectionController, CollectionState } from "./collection-model";
import type { MediaId, MediaItem } from "./contracts";
import { userErrorMessage } from "./presentation";

type MutationActions = Pick<
  CollectionController,
  "importPaths" | "toggleFavorite" | "showOverlay" | "rename" | "replaceTags" | "remove"
>;

interface MutationContext {
  readonly nativeAvailable: boolean;
  readonly getState: () => CollectionState;
  readonly updateState: (updater: (state: CollectionState) => CollectionState) => void;
  readonly reload: () => Promise<void>;
  readonly notify: CollectionController["notify"];
}

export function createCollectionMutations(context: MutationContext): MutationActions {
  function setBusy(id: MediaId, busy: boolean): void {
    context.updateState((state) => {
      const busyIds = new Set(state.busyIds);
      if (busy) busyIds.add(id);
      else busyIds.delete(id);
      return { ...state, busyIds };
    });
  }

  function replaceItem(item: MediaItem): void {
    context.updateState((state) => ({
      ...state,
      items: state.items.map((current) => (current.id === item.id ? item : current)),
    }));
  }

  async function importPaths(paths: readonly string[]): Promise<void> {
    if (paths.length === 0) return;
    if (!context.nativeAvailable) {
      context.notify("info", "File import is available in the LumaDesk desktop app.");
      return;
    }
    context.updateState((state) => ({ ...state, importing: true, dragActive: false }));
    try {
      const report = await importMedia(paths);
      const selectedId = report.outcomes[0]?.item.id ?? null;
      context.updateState((state) => ({ ...state, importReport: report, selectedId }));
      await context.reload();
      const imported = report.outcomes.filter((outcome) => outcome.status === "imported").length;
      const duplicate = report.outcomes.length - imported;
      context.notify(
        "success",
        `${imported} new, ${duplicate} duplicate, ${report.failures.length} failed.`,
      );
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "The files could not be imported.")
          : "The files could not be imported.";
      context.notify("error", message);
    } finally {
      context.updateState((state) => ({ ...state, importing: false }));
    }
  }

  async function toggleFavorite(item: MediaItem): Promise<void> {
    if (!context.nativeAvailable) return;
    setBusy(item.id, true);
    try {
      const updated = await setFavorite(item.id, !item.isFavorite);
      if (context.getState().filters.favoriteOnly && item.isFavorite) await context.reload();
      else replaceItem(updated);
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "The favorite status could not be changed.")
          : "The favorite status could not be changed.";
      context.notify("error", message);
    } finally {
      setBusy(item.id, false);
    }
  }

  async function showOverlay(item: MediaItem): Promise<void> {
    if (!context.nativeAvailable) {
      context.notify("info", "Desktop overlays are available in the LumaDesk desktop app.");
      return;
    }
    setBusy(item.id, true);
    try {
      await showMediaOverlay(item.id);
      context.notify("success", "Animation shown on the desktop.");
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "The desktop overlay could not be opened.")
          : "The desktop overlay could not be opened.";
      context.notify("error", message);
    } finally {
      setBusy(item.id, false);
    }
  }

  async function rename(item: MediaItem, name: string): Promise<boolean> {
    if (!context.nativeAvailable) return false;
    setBusy(item.id, true);
    try {
      const updated = await renameMedia(item.id, name);
      if (context.getState().filters.search.trim() === "") replaceItem(updated);
      else await context.reload();
      context.notify("success", "Animation name updated.");
      return true;
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "The name could not be changed.")
          : "The name could not be changed.";
      context.notify("error", message);
      return false;
    } finally {
      setBusy(item.id, false);
    }
  }

  async function updateTags(item: MediaItem, tags: readonly string[]): Promise<boolean> {
    if (!context.nativeAvailable) return false;
    setBusy(item.id, true);
    try {
      const updated = await replaceTags(item.id, tags);
      const filters = context.getState().filters;
      if (filters.search.trim() === "" && filters.tag.trim() === "") replaceItem(updated);
      else await context.reload();
      context.notify("success", "Animation tags updated.");
      return true;
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "The tags could not be changed.")
          : "The tags could not be changed.";
      context.notify("error", message);
      return false;
    } finally {
      setBusy(item.id, false);
    }
  }

  async function remove(item: MediaItem): Promise<boolean> {
    if (!context.nativeAvailable) return false;
    setBusy(item.id, true);
    try {
      const report = await deleteMedia(item.id);
      await context.reload();
      const message =
        report.overlaysRemoved > 0
          ? `Animation removed with ${report.overlaysRemoved} active overlays.`
          : "Animation removed from the collection.";
      context.notify("success", message);
      return true;
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "The animation could not be removed.")
          : "The animation could not be removed.";
      context.notify("error", message);
      return false;
    } finally {
      setBusy(item.id, false);
    }
  }

  return { importPaths, toggleFavorite, showOverlay, rename, replaceTags: updateTags, remove };
}
