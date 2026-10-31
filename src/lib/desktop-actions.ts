import { get, type Writable } from "svelte/store";
import {
  applyDesktopScene,
  createDesktopScene,
  deleteDesktopScene,
  duplicateDesktopScene,
  duplicateOverlay,
  moveOverlayToMonitor,
  patchAllOverlays,
  removeOverlay,
  renameDesktopScene,
  updateDesktopScene,
  updateOverlay,
} from "./api";
import type { ToastTone } from "./collection-model";
import type { DesktopSceneId, OverlayBatchPatch, OverlayId, OverlayPatch } from "./contracts";
import { runSettledBatch } from "./desktop-batch";
import type { DesktopController, DesktopState } from "./desktop-model";
import { userErrorMessage } from "./presentation";

type DesktopActions = Pick<
  DesktopController,
  | "update"
  | "moveToMonitor"
  | "duplicate"
  | "close"
  | "showAll"
  | "hideAll"
  | "playAll"
  | "pauseAll"
  | "lockAll"
  | "unlockAll"
  | "closeAll"
  | "createScene"
  | "applyScene"
  | "updateScene"
  | "renameScene"
  | "duplicateScene"
  | "deleteScene"
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

  async function moveToMonitor(id: OverlayId, monitorName: string): Promise<void> {
    if (!nativeAvailable) return;
    setBusy([id], true);
    try {
      replaceOverlay(await moveOverlayToMonitor(id, monitorName));
      notify("success", `Overlay moved to ${monitorName}.`);
    } catch (error) {
      actionError(error, "The overlay could not be moved to that display.");
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

  async function performCloseBatch(
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

  async function applyGlobalPatch(
    patch: OverlayBatchPatch,
    successMessage: string,
    fallback: string,
  ): Promise<void> {
    if (!nativeAvailable) return;
    store.update((state) => ({ ...state, batchBusy: true }));
    try {
      await patchAllOverlays(patch);
      await reload();
      notify("success", successMessage);
    } catch (error) {
      actionError(error, fallback);
    } finally {
      store.update((state) => ({ ...state, batchBusy: false }));
    }
  }

  async function showAll(): Promise<void> {
    await applyGlobalPatch(
      { isActive: true },
      "All overlays are visible.",
      "Overlays could not be shown.",
    );
  }

  async function hideAll(): Promise<void> {
    await applyGlobalPatch(
      { isActive: false },
      "All overlays are hidden.",
      "Overlays could not be hidden.",
    );
  }

  async function playAll(): Promise<void> {
    await applyGlobalPatch(
      { isPlaying: true },
      "All overlays are playing.",
      "Overlays could not be started.",
    );
  }

  async function pauseAll(): Promise<void> {
    await applyGlobalPatch(
      { isPlaying: false },
      "All overlays are paused.",
      "Overlays could not be paused.",
    );
  }

  async function lockAll(): Promise<void> {
    await applyGlobalPatch(
      { clickThrough: true },
      "All overlays are click-through.",
      "Overlays could not be locked.",
    );
  }

  async function unlockAll(): Promise<void> {
    await applyGlobalPatch(
      { clickThrough: false },
      "All overlays are interactive.",
      "Overlays could not be unlocked.",
    );
  }

  async function closeAll(): Promise<void> {
    const ids = get(store).overlays.map((item) => item.overlay.id);
    if (ids.length === 0) return;
    await performCloseBatch(
      ids,
      removeOverlay,
      `${ids.length} overlays closed.`,
      (failed) => `${failed} overlays could not be closed.`,
    );
  }

  async function runSceneAction(
    action: () => Promise<unknown>,
    successMessage: string,
    fallback: string,
  ): Promise<void> {
    if (!nativeAvailable) return;
    store.update((state) => ({ ...state, sceneBusy: true }));
    try {
      await action();
      await reload();
      notify("success", successMessage);
    } catch (error) {
      actionError(error, fallback);
    } finally {
      store.update((state) => ({ ...state, sceneBusy: false }));
    }
  }

  async function createScene(name: string): Promise<void> {
    await runSceneAction(
      () => createDesktopScene(name),
      `Scene “${name.trim()}” saved.`,
      "The scene could not be saved.",
    );
  }

  async function applyScene(id: DesktopSceneId): Promise<void> {
    const name = get(store).scenes.find((scene) => scene.id === id)?.name ?? "Scene";
    await runSceneAction(
      () => applyDesktopScene(id),
      `Scene “${name}” applied.`,
      "The scene could not be applied.",
    );
  }

  async function updateScene(id: DesktopSceneId): Promise<void> {
    await runSceneAction(
      () => updateDesktopScene(id),
      "Scene snapshot updated.",
      "The scene snapshot could not be updated.",
    );
  }

  async function renameScene(id: DesktopSceneId, name: string): Promise<void> {
    await runSceneAction(
      () => renameDesktopScene(id, name),
      `Scene renamed to “${name.trim()}”.`,
      "The scene could not be renamed.",
    );
  }

  async function duplicateScene(id: DesktopSceneId): Promise<void> {
    await runSceneAction(
      () => duplicateDesktopScene(id),
      "Scene duplicated.",
      "The scene could not be duplicated.",
    );
  }

  async function deleteScene(id: DesktopSceneId): Promise<void> {
    await runSceneAction(
      () => deleteDesktopScene(id),
      "Scene deleted.",
      "The scene could not be deleted.",
    );
  }

  return {
    update,
    moveToMonitor,
    duplicate,
    close,
    showAll,
    hideAll,
    playAll,
    pauseAll,
    lockAll,
    unlockAll,
    closeAll,
    createScene,
    applyScene,
    updateScene,
    renameScene,
    duplicateScene,
    deleteScene,
  };
}
