<script lang="ts">
import { isTauri } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { onMount } from "svelte";
import { createCollectionController } from "./lib/collection-controller";
import type { CollectionDialog } from "./lib/collection-model";
import AppChrome from "./lib/components/AppChrome.svelte";
import CollectionDialogs from "./lib/components/CollectionDialogs.svelte";
import CollectionWorkspace from "./lib/components/CollectionWorkspace.svelte";
import DesktopWorkspace from "./lib/components/DesktopWorkspace.svelte";
import MediaInspector from "./lib/components/MediaInspector.svelte";
import Toast from "./lib/components/Toast.svelte";
import type { MediaItem } from "./lib/contracts";
import { createDesktopController } from "./lib/desktop-controller";
import { gifDiscoveryUrl } from "./lib/gif-discovery";
import type { NavigationId } from "./lib/navigation";
import { userErrorMessage } from "./lib/presentation";

const nativeAvailable = isTauri();
const collection = createCollectionController(nativeAvailable);
const desktop = createDesktopController(nativeAvailable);
let dialog: CollectionDialog | null = null;
let activePage: NavigationId = "collection";
$: selectedItem =
  $collection.selectedId === null
    ? undefined
    : $collection.items.find((item) => item.id === $collection.selectedId);
$: selectedBusy = selectedItem === undefined ? false : $collection.busyIds.has(selectedItem.id);
$: pageTitle = activePage === "desktop" ? "Active on Desktop" : "Collection";

onMount(() => {
  let disposed = false;
  let unlisten: UnlistenFn | undefined;
  void collection.initialize();
  void desktop.initialize();

  if (nativeAvailable) {
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "drop") {
          collection.setDragActive(false);
          void collection.importPaths(event.payload.paths);
        } else if (event.payload.type === "leave") {
          collection.setDragActive(false);
        } else {
          collection.setDragActive(true);
        }
      })
      .then((cleanup) => {
        if (disposed) cleanup();
        else unlisten = cleanup;
      })
      .catch((error) => {
        const message =
          error instanceof Error
            ? userErrorMessage(error, "Drag and drop could not be enabled.")
            : "Drag and drop could not be enabled.";
        collection.notify("error", message);
      });
  }

  return () => {
    disposed = true;
    unlisten?.();
    collection.destroy();
    desktop.destroy();
  };
});

async function chooseFiles(): Promise<void> {
  if (!nativeAvailable) {
    collection.notify("info", "The file picker is available in the LumaDesk desktop app.");
    return;
  }
  try {
    const paths = await open({
      title: "Import animations into LumaDesk",
      multiple: true,
      directory: false,
      filters: [
        {
          name: "Animations and images",
          extensions: ["gif", "apng", "webp", "png", "jpg", "jpeg"],
        },
      ],
    });
    if (paths !== null) await collection.importPaths(paths);
  } catch (error) {
    const message =
      error instanceof Error
        ? userErrorMessage(error, "The file picker could not be opened.")
        : "The file picker could not be opened.";
    collection.notify("error", message);
  }
}

async function discoverGifs(): Promise<void> {
  const url = gifDiscoveryUrl($collection.filters.search);
  if (!nativeAvailable) {
    const opened = window.open(url, "_blank", "noopener,noreferrer");
    if (opened === null) {
      collection.notify(
        "error",
        "Your browser blocked the GIF search page. Allow pop-ups and try again.",
      );
    }
    return;
  }

  try {
    await openUrl(url);
  } catch (error) {
    const message =
      error instanceof Error
        ? userErrorMessage(error, "The GIF search page could not be opened.")
        : "The GIF search page could not be opened.";
    collection.notify("error", message);
  }
}

function edit(kind: CollectionDialog["kind"], item: MediaItem): void {
  dialog = { kind, item };
}

function navigate(id: NavigationId, enabled: boolean): void {
  if (enabled) activePage = id;
}
</script>

<svelte:head><title>LumaDesk — {pageTitle}</title></svelte:head>

<div class="app-shell">
  <AppChrome
    {activePage}
    collectionCount={$collection.total}
    desktopCount={$desktop.overlays.length}
    onnavigate={navigate}
  />

  <div class:with-inspector={activePage === "collection" && selectedItem !== undefined} class="workspace-region">
    {#if activePage === "collection"}
      <CollectionWorkspace
        state={$collection}
        onsearch={collection.setSearch}
        onkind={collection.setMediaKind}
        onfavoritefilter={collection.setFavoriteOnly}
        ontag={collection.setTag}
        onview={(view) => void collection.setView(view)}
        ondiscover={() => void discoverGifs()}
        onimport={() => void chooseFiles()}
        onselect={collection.select}
        onfavorite={(item) => void collection.toggleFavorite(item)}
        onpage={collection.setPage}
        onreload={() => void collection.reload()}
      />

      {#if selectedItem !== undefined}
        <MediaInspector
          item={selectedItem}
          src={$collection.assetUrls[selectedItem.id]}
          busy={selectedBusy}
          onclose={() => collection.select(null)}
          onrename={(item) => edit("rename", item)}
          ontags={(item) => edit("tags", item)}
          ondelete={(item) => edit("delete", item)}
          onfavorite={(item) => void collection.toggleFavorite(item)}
          onshow={(item) => void collection.showOverlay(item)}
        />
      {/if}
    {:else if activePage === "desktop"}
      <DesktopWorkspace
        state={$desktop}
        onupdate={(id, patch) => void desktop.update(id, patch)}
        onduplicate={(id) => void desktop.duplicate(id)}
        onclose={(id) => void desktop.close(id)}
        onpauseall={() => void desktop.pauseAll()}
        oncloseall={() => void desktop.closeAll()}
        onreload={() => void desktop.reload()}
        oncollection={() => (activePage = "collection")}
      />
    {/if}
  </div>
</div>

<CollectionDialogs
  {dialog}
  report={$collection.importReport}
  busy={selectedBusy}
  onclose={() => (dialog = null)}
  onclosereport={collection.closeImportReport}
  onrename={collection.rename}
  ontags={collection.replaceTags}
  ondelete={collection.remove}
/>

{#if activePage === "collection" && $collection.notice !== null}
  {#key $collection.notice.id}
    <Toast message={$collection.notice.message} tone={$collection.notice.tone} ondismiss={collection.dismissNotice} />
  {/key}
{:else if activePage === "desktop" && $desktop.notice !== null}
  {#key $desktop.notice.id}
    <Toast message={$desktop.notice.message} tone={$desktop.notice.tone} ondismiss={desktop.dismissNotice} />
  {/key}
{/if}

<style>
.app-shell {
  display: grid;
  grid-template: 40px minmax(0, 1fr) / 224px minmax(0, 1fr);
  width: 100%;
  height: 100dvh;
  overflow: hidden;
  background: var(--canvas-950);
}

.workspace-region { position: relative; display: grid; min-width: 0; min-height: 0; grid-template-columns: minmax(0, 1fr); }
.workspace-region.with-inspector { grid-template-columns: minmax(0, 1fr) 320px; }

@media (max-width: 1100px) {
  .workspace-region.with-inspector { grid-template-columns: minmax(0, 1fr); }
}

@media (max-width: 760px) {
  .app-shell { grid-template: 40px minmax(0, 1fr) 64px / 1fr; }
  .workspace-region { grid-row: 2; }
}
</style>
