import { get, type Writable } from "svelte/store";
import { duplicateOverlay, removeOverlay, updateOverlay } from "./api";
import type { ToastTone } from "./collection-model";
import type { OverlayId, OverlayPatch } from "./contracts";
import { runSettledBatch } from "./desktop-batch";
import type { DesktopController, DesktopState } from "./desktop-model";
import { userErrorMessage } from "./presentation";

type DesktopActions = Pick<
  DesktopController,
  "update" | "duplicate" | "close" | "pauseAll" | "closeAll"
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

  async function update(id: OverlayId, patch: OverlayPatch): Promise<void> {
    if (!nativeAvailable) return;
    setBusy([id], true);
    try {
      replaceOverlay(await updateOverlay(id, patch));
    } catch (error) {
      actionError(error, "The overlay settings could not be saved.");
    } finally {
      setBusy([id], false);
    }
  }

  async function duplicate(id: OverlayId): Promise<void> {
    if (!nativeAvailable) return;
    setBusy([id], true);
    try {
      await duplicateOverlay(id);
      notify("success", "Overlay duplicated.");
      await reload();
    } catch (error) {
      actionError(error, "The overlay could not be duplicated.");
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
      notify("success", "Overlay closed.");
    } catch (error) {
      actionError(error, "The overlay could not be closed.");
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
      notify("info", "All overlays are already paused.");
      return;
    }
    await performBatch(
      ids,
      (id) => updateOverlay(id, { isPlaying: false }),
      `${ids.length} overlays paused.`,
      (failed) => `${failed} overlays could not be paused.`,
    );
  }

  async function closeAll(): Promise<void> {
    const ids = get(store).overlays.map((item) => item.overlay.id);
    if (ids.length === 0) return;
    await performBatch(
      ids,
      removeOverlay,
      `${ids.length} overlays closed.`,
      (failed) => `${failed} overlays could not be closed.`,
    );
  }

  return { update, duplicate, close, pauseAll, closeAll };
}
