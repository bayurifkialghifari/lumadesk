import { get, type Writable } from "svelte/store";
import { duplicateOverlay, removeOverlay, setOverlayEditMode, updateOverlay } from "./api";
import type { ToastTone } from "./collection-model";
import type { OverlayId, OverlayPatch } from "./contracts";
import { runSettledBatch } from "./desktop-batch";
import type { DesktopController, DesktopState } from "./desktop-model";
import { userErrorMessage } from "./presentation";

type DesktopActions = Pick<
  DesktopController,
  "setEditMode" | "update" | "duplicate" | "close" | "pauseAll" | "closeAll"
>;

interface DesktopActionContext {
  readonly nativeAvailable: boolean;
  readonly store: Writable<DesktopState>;
  readonly reload: () => Promise<void>;
  readonly notify: (tone: ToastTone, message: string) => void;
}

export function createDesktopActions(context: DesktopActionContext): DesktopActions {
  const { nativeAvailable, store, reload, notify } = context;

  function setBusy(ids: readonly OverlayId[], busy: boolean): void {
    store.update((state) => {
      const busyIds = new Set(state.busyIds);
      for (const id of ids) {
        if (busy) busyIds.add(id);
        else busyIds.delete(id);
      }
      return { ...state, busyIds };
    });
  }

  function replaceOverlay(overlay: DesktopState["overlays"][number]["overlay"]): void {
    store.update((state) => ({
      ...state,
      overlays: state.overlays.map((item) =>
        item.overlay.id === overlay.id ? { ...item, overlay } : item,
      ),
    }));
  }

  function actionError(error: unknown, fallback: string): void {
    notify("error", error instanceof Error ? userErrorMessage(error, fallback) : fallback);
  }

  async function setEditMode(enabled: boolean): Promise<void> {
    if (!nativeAvailable) {
      notify("info", "Mode edit tersedia saat LumaDesk dijalankan sebagai aplikasi desktop.");
      return;
    }
    store.update((state) => ({ ...state, editModeBusy: true }));
    try {
      const editMode = await setOverlayEditMode(enabled);
      store.update((state) => ({ ...state, editMode }));
    } catch (error) {
      actionError(error, "Mode edit tidak dapat diubah.");
    } finally {
      store.update((state) => ({ ...state, editModeBusy: false }));
    }
  }

  async function update(id: OverlayId, patch: OverlayPatch): Promise<void> {
    if (!nativeAvailable) return;
    setBusy([id], true);
    try {
      replaceOverlay(await updateOverlay(id, patch));
    } catch (error) {
      actionError(error, "Pengaturan overlay tidak dapat disimpan.");
    } finally {
      setBusy([id], false);
    }
  }

  async function duplicate(id: OverlayId): Promise<void> {
    if (!nativeAvailable) return;
    setBusy([id], true);
    try {
      await duplicateOverlay(id);
      notify("success", "Overlay diduplikasi.");
      await reload();
    } catch (error) {
      actionError(error, "Overlay tidak dapat diduplikasi.");
    } finally {
      setBusy([id], false);
    }
  }

  async function close(id: OverlayId): Promise<void> {
    if (!nativeAvailable) return;
    setBusy([id], true);
    try {
      await removeOverlay(id);
      store.update((state) => ({
        ...state,
        overlays: state.overlays.filter((item) => item.overlay.id !== id),
      }));
      notify("success", "Overlay ditutup.");
    } catch (error) {
      actionError(error, "Overlay tidak dapat ditutup.");
    } finally {
      setBusy([id], false);
    }
  }

  async function performBatch(
    ids: readonly OverlayId[],
    operation: (id: OverlayId) => Promise<unknown>,
    successMessage: string,
    failureMessage: (failed: number) => string,
  ): Promise<void> {
    store.update((state) => ({ ...state, batchBusy: true }));
    setBusy(ids, true);
    const outcome = await runSettledBatch(ids, operation);
    await reload();
    setBusy(ids, false);
    store.update((state) => ({ ...state, batchBusy: false }));
    notify(
      outcome.failed === 0 ? "success" : "error",
      outcome.failed === 0 ? successMessage : failureMessage(outcome.failed),
    );
  }

  async function pauseAll(): Promise<void> {
    const ids = get(store)
      .overlays.filter((item) => item.overlay.isPlaying)
      .map((item) => item.overlay.id);
    if (ids.length === 0) {
      notify("info", "Semua overlay sudah dijeda.");
      return;
    }
    await performBatch(
      ids,
      (id) => updateOverlay(id, { isPlaying: false }),
      `${ids.length} overlay dijeda.`,
      (failed) => `${failed} overlay tidak dapat dijeda.`,
    );
  }

  async function closeAll(): Promise<void> {
    const ids = get(store).overlays.map((item) => item.overlay.id);
    if (ids.length === 0) return;
    await performBatch(
      ids,
      removeOverlay,
      `${ids.length} overlay ditutup.`,
      (failed) => `${failed} overlay tidak dapat ditutup.`,
    );
  }

  return { setEditMode, update, duplicate, close, pauseAll, closeAll };
}
