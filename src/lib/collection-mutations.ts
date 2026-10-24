import { deleteMedia, importMedia, renameMedia, replaceTags, setFavorite } from "./api";
import type { CollectionController, CollectionState } from "./collection-model";
import type { MediaId, MediaItem } from "./contracts";
import { userErrorMessage } from "./presentation";

type MutationActions = Pick<
  CollectionController,
  "importPaths" | "toggleFavorite" | "rename" | "replaceTags" | "remove"
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
      context.notify(
        "info",
        "Impor file tersedia saat LumaDesk dijalankan sebagai aplikasi desktop.",
      );
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
        `${imported} baru, ${duplicate} duplikat, ${report.failures.length} gagal.`,
      );
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "File tidak dapat diimpor.")
          : "File tidak dapat diimpor.";
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
          ? userErrorMessage(error, "Favorit tidak dapat diubah.")
          : "Favorit tidak dapat diubah.";
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
      context.notify("success", "Nama animasi diperbarui.");
      return true;
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "Nama tidak dapat diubah.")
          : "Nama tidak dapat diubah.";
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
      context.notify("success", "Tag animasi diperbarui.");
      return true;
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "Tag tidak dapat diubah.")
          : "Tag tidak dapat diubah.";
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
          ? `Animasi dihapus bersama ${report.overlaysRemoved} overlay aktif.`
          : "Animasi dihapus dari koleksi.";
      context.notify("success", message);
      return true;
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "Animasi tidak dapat dihapus.")
          : "Animasi tidak dapat dihapus.";
      context.notify("error", message);
      return false;
    } finally {
      setBusy(item.id, false);
    }
  }

  return { importPaths, toggleFavorite, rename, replaceTags: updateTags, remove };
}
