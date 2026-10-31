import { get, writable } from "svelte/store";
import { getEditRecipe, getSettings, listMedia, mediaAssetUrl, updateSettings } from "./api";
import {
  ALL_MEDIA_FORMATS,
  buildLibraryQuery,
  type CollectionController,
  type CollectionState,
  DEFAULT_SETTINGS,
  lastCollectionPage,
  type MediaFormatFilter,
  type ToastTone,
} from "./collection-model";
import { createCollectionMutations } from "./collection-mutations";
import type { EditRecipe, MediaItem } from "./contracts";
import { userErrorMessage } from "./presentation";

const SEARCH_DELAY_MS = 180;
const NOTICE_DURATION_MS = 5_000;

export function createCollectionController(nativeAvailable: boolean): CollectionController {
  const store = writable<CollectionState>({
    nativeAvailable,
    ready: false,
    loading: false,
    refreshing: false,
    importing: false,
    dragActive: false,
    items: [],
    total: 0,
    page: 0,
    filters: { search: "", mediaFormat: ALL_MEDIA_FORMATS, favoriteOnly: false, tag: "" },
    settings: DEFAULT_SETTINGS,
    selectedId: null,
    assetUrls: {},
    editRecipes: {},
    busyIds: new Set(),
    loadError: null,
    notice: null,
    importReport: null,
  });
  let requestSequence = 0;
  let noticeSequence = 0;
  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  let noticeTimer: ReturnType<typeof setTimeout> | undefined;

  function dismissNotice(): void {
    if (noticeTimer !== undefined) clearTimeout(noticeTimer);
    noticeTimer = undefined;
    store.update((state) => ({ ...state, notice: null }));
  }

  function notify(tone: ToastTone, message: string): void {
    if (noticeTimer !== undefined) clearTimeout(noticeTimer);
    noticeSequence += 1;
    store.update((state) => ({ ...state, notice: { id: noticeSequence, tone, message } }));
    noticeTimer = setTimeout(dismissNotice, NOTICE_DURATION_MS);
  }

  async function resolveAssetUrls(
    items: readonly MediaItem[],
  ): Promise<Readonly<Record<string, string>>> {
    const results = await Promise.allSettled(
      items.map(async (item) => ({ id: item.id, url: await mediaAssetUrl(item.id) })),
    );
    const urls: Record<string, string> = {};
    for (const result of results) {
      if (result.status === "fulfilled") urls[result.value.id] = result.value.url;
    }
    return urls;
  }

  async function resolveEditRecipes(
    items: readonly MediaItem[],
  ): Promise<Readonly<Record<string, EditRecipe>>> {
    const results = await Promise.allSettled(
      items.map(async (item) => ({ id: item.id, recipe: await getEditRecipe(item.id) })),
    );
    const recipes: Record<string, EditRecipe> = {};
    for (const result of results) {
      if (result.status === "fulfilled") recipes[result.value.id] = result.value.recipe;
    }
    return recipes;
  }

  async function reload(): Promise<void> {
    if (!nativeAvailable) {
      store.update((state) => ({ ...state, ready: true, loading: false, refreshing: false }));
      return;
    }
    requestSequence += 1;
    const request = requestSequence;
    const snapshot = get(store);
    store.update((state) => ({
      ...state,
      loading: !state.ready,
      refreshing: state.ready,
      loadError: null,
    }));
    try {
      const page = await listMedia(buildLibraryQuery(snapshot));
      const lastPage = lastCollectionPage(page.total);
      if (snapshot.page > lastPage) {
        if (request !== requestSequence) return;
        store.update((state) => ({ ...state, page: lastPage }));
        await reload();
        return;
      }
      const [assetUrls, editRecipes] = await Promise.all([
        resolveAssetUrls(page.items),
        resolveEditRecipes(page.items),
      ]);
      if (request !== requestSequence) return;
      store.update((state) => {
        const selectedId = page.items.some((item) => item.id === state.selectedId)
          ? state.selectedId
          : null;
        return {
          ...state,
          ready: true,
          loading: false,
          refreshing: false,
          items: page.items,
          total: page.total,
          selectedId,
          assetUrls,
          editRecipes,
        };
      });
    } catch (error) {
      if (request !== requestSequence) return;
      const message =
        error instanceof Error
          ? userErrorMessage(error, "The collection could not be loaded.")
          : "The collection could not be loaded.";
      store.update((state) => ({
        ...state,
        ready: true,
        loading: false,
        refreshing: false,
        loadError: message,
      }));
    }
  }

  async function initialize(): Promise<void> {
    if (!nativeAvailable) {
      await reload();
      return;
    }
    try {
      const settings = await getSettings();
      store.update((state) => ({ ...state, settings }));
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "Local settings could not be loaded.")
          : "Local settings could not be loaded.";
      notify("error", message);
    }
    await reload();
  }

  function scheduleReload(): void {
    if (searchTimer !== undefined) clearTimeout(searchTimer);
    requestSequence += 1;
    searchTimer = setTimeout(() => {
      searchTimer = undefined;
      void reload();
    }, SEARCH_DELAY_MS);
  }

  function updateFilter(update: Partial<CollectionState["filters"]>, delayed = false): void {
    store.update((state) => ({
      ...state,
      page: 0,
      filters: { ...state.filters, ...update },
    }));
    if (delayed) scheduleReload();
    else {
      if (searchTimer !== undefined) clearTimeout(searchTimer);
      searchTimer = undefined;
      void reload();
    }
  }

  async function setView(view: CollectionState["settings"]["collectionView"]): Promise<void> {
    const previous = get(store).settings;
    const next = { ...previous, collectionView: view };
    store.update((state) => ({ ...state, settings: next }));
    if (!nativeAvailable) return;
    try {
      const settings = await updateSettings(next);
      store.update((state) => ({ ...state, settings }));
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "The collection view could not be saved.")
          : "The collection view could not be saved.";
      store.update((state) => ({ ...state, settings: previous }));
      notify("error", message);
    }
  }

  async function saveSettings(next: CollectionState["settings"]): Promise<boolean> {
    const previous = get(store).settings;
    store.update((state) => ({ ...state, settings: next }));
    if (!nativeAvailable) return true;
    try {
      const settings = await updateSettings(next);
      store.update((state) => ({ ...state, settings }));
      notify("success", "Settings saved.");
      return true;
    } catch (error) {
      const message =
        error instanceof Error
          ? userErrorMessage(error, "Settings could not be saved.")
          : "Settings could not be saved.";
      store.update((state) => ({ ...state, settings: previous }));
      notify("error", message);
      return false;
    }
  }

  const mutations = createCollectionMutations({
    nativeAvailable,
    getState: () => get(store),
    updateState: store.update,
    reload,
    notify,
  });

  return {
    subscribe: store.subscribe,
    initialize,
    reload,
    setSearch: (search) => updateFilter({ search }, true),
    setMediaFormat: (mediaFormat: MediaFormatFilter) => updateFilter({ mediaFormat }),
    setFavoriteOnly: (favoriteOnly) => updateFilter({ favoriteOnly }),
    setTag: (tag) => updateFilter({ tag }),
    setPage: (page) => {
      store.update((state) => ({ ...state, page }));
      void reload();
    },
    setView,
    saveSettings,
    select: (selectedId) => store.update((state) => ({ ...state, selectedId })),
    setDragActive: (dragActive) => store.update((state) => ({ ...state, dragActive })),
    ...mutations,
    notify,
    dismissNotice,
    closeImportReport: () => store.update((state) => ({ ...state, importReport: null })),
    destroy: () => {
      if (searchTimer !== undefined) clearTimeout(searchTimer);
      if (noticeTimer !== undefined) clearTimeout(noticeTimer);
    },
  };
}
