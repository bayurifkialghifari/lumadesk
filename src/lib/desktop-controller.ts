import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import { listDesktopOverlays, mediaAssetUrl } from "./api";
import type { ToastTone } from "./collection-model";
import type { DesktopOverlay } from "./contracts";
import { createDesktopActions } from "./desktop-actions";
import type { DesktopController, DesktopState } from "./desktop-model";
import { userErrorMessage } from "./presentation";

const OVERLAY_CATALOG_EVENT = "overlay:catalog-changed";
const NOTICE_DURATION_MS = 5_000;
const RELOAD_DELAY_MS = 100;

export function createDesktopController(nativeAvailable: boolean): DesktopController {
  const store = writable<DesktopState>({
    nativeAvailable,
    ready: false,
    loading: false,
    refreshing: false,
    overlays: [],
    assetUrls: {},
    busyIds: new Set(),
    batchBusy: false,
    loadError: null,
    notice: null,
  });
  let requestSequence = 0;
  let noticeSequence = 0;
  let noticeTimer: ReturnType<typeof setTimeout> | undefined;
  let reloadTimer: ReturnType<typeof setTimeout> | undefined;
  let unlistenCatalog: UnlistenFn | undefined;
  let disposed = false;

  function dismissNotice(): void {
    if (noticeTimer !== undefined) clearTimeout(noticeTimer);
    noticeTimer = undefined;
    store.update((state) => ({ ...state, notice: null }));
  }

  function notify(tone: ToastTone, message: string): void {
    if (disposed) return;
    if (noticeTimer !== undefined) clearTimeout(noticeTimer);
    noticeSequence += 1;
    store.update((state) => ({ ...state, notice: { id: noticeSequence, tone, message } }));
    noticeTimer = setTimeout(dismissNotice, NOTICE_DURATION_MS);
  }

  async function resolveAssetUrls(
    overlays: readonly DesktopOverlay[],
  ): Promise<Readonly<Record<string, string>>> {
    const mediaIds = [...new Set(overlays.map((item) => item.media.id))];
    const results = await Promise.allSettled(
      mediaIds.map(async (id) => ({ id, url: await mediaAssetUrl(id) })),
    );
    const urls: Record<string, string> = {};
    for (const result of results) {
      if (result.status === "fulfilled") urls[result.value.id] = result.value.url;
    }
    return urls;
  }

  async function reload(): Promise<void> {
    if (!nativeAvailable) {
      store.update((state) => ({ ...state, ready: true, loading: false, refreshing: false }));
      return;
    }
    requestSequence += 1;
    const request = requestSequence;
    store.update((state) => ({
      ...state,
      loading: !state.ready,
      refreshing: state.ready,
      loadError: null,
    }));
    try {
      const overlays = await listDesktopOverlays();
      const assetUrls = await resolveAssetUrls(overlays);
      if (disposed || request !== requestSequence) return;
      store.update((state) => ({
        ...state,
        ready: true,
        loading: false,
        refreshing: false,
        overlays,
        assetUrls,
      }));
    } catch (error) {
      if (disposed || request !== requestSequence) return;
      const message =
        error instanceof Error
          ? userErrorMessage(error, "Active overlays could not be loaded.")
          : "Active overlays could not be loaded.";
      store.update((state) => ({
        ...state,
        ready: true,
        loading: false,
        refreshing: false,
        loadError: message,
      }));
    }
  }

  function scheduleReload(): void {
    if (reloadTimer !== undefined) clearTimeout(reloadTimer);
    reloadTimer = setTimeout(() => {
      reloadTimer = undefined;
      void reload();
    }, RELOAD_DELAY_MS);
  }

  async function initialize(): Promise<void> {
    if (!nativeAvailable) {
      await reload();
      return;
    }
    try {
      const cleanup = await listen<void>(OVERLAY_CATALOG_EVENT, scheduleReload);
      if (disposed) cleanup();
      else unlistenCatalog = cleanup;
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "Overlay synchronization could not be enabled.")
          : "Overlay synchronization could not be enabled.";
      notify("error", message);
    }
    await reload();
  }

  const actions = createDesktopActions({ nativeAvailable, store, reload, notify });

  return {
    subscribe: store.subscribe,
    initialize,
    reload,
    ...actions,
    notify,
    dismissNotice,
    destroy: () => {
      disposed = true;
      requestSequence += 1;
      if (noticeTimer !== undefined) clearTimeout(noticeTimer);
      if (reloadTimer !== undefined) clearTimeout(reloadTimer);
      unlistenCatalog?.();
    },
  };
}
