<script lang="ts">
import { Sparkles } from "@lucide/svelte";
import { isTauri } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { onMount } from "svelte";
import { createCollectionController } from "./lib/collection-controller";
import type { CollectionDialog } from "./lib/collection-model";
import CollectionDialogs from "./lib/components/CollectionDialogs.svelte";
import CollectionWorkspace from "./lib/components/CollectionWorkspace.svelte";
import MediaInspector from "./lib/components/MediaInspector.svelte";
import Toast from "./lib/components/Toast.svelte";
import type { MediaItem } from "./lib/contracts";
import { navigation } from "./lib/navigation";
import { userErrorMessage } from "./lib/presentation";

const nativeAvailable = isTauri();
const collection = createCollectionController(nativeAvailable);
let dialog: CollectionDialog | null = null;
$: selectedItem =
  $collection.selectedId === null
    ? undefined
    : $collection.items.find((item) => item.id === $collection.selectedId);
$: selectedBusy = selectedItem === undefined ? false : $collection.busyIds.has(selectedItem.id);

onMount(() => {
  let disposed = false;
  let unlisten: UnlistenFn | undefined;
  void collection.initialize();

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
            ? userErrorMessage(error, "Drag dan drop tidak dapat diaktifkan.")
            : "Drag dan drop tidak dapat diaktifkan.";
        collection.notify("error", message);
      });
  }

  return () => {
    disposed = true;
    unlisten?.();
    collection.destroy();
  };
});

async function chooseFiles(): Promise<void> {
  if (!nativeAvailable) {
    collection.notify(
      "info",
      "Pemilih file tersedia saat LumaDesk dijalankan sebagai aplikasi desktop.",
    );
    return;
  }
  try {
    const paths = await open({
      title: "Impor animasi ke LumaDesk",
      multiple: true,
      directory: false,
      filters: [
        { name: "Animasi dan gambar", extensions: ["gif", "apng", "webp", "png", "jpg", "jpeg"] },
      ],
    });
    if (paths !== null) await collection.importPaths(paths);
  } catch (error) {
    const message =
      error instanceof Error
        ? userErrorMessage(error, "Pemilih file tidak dapat dibuka.")
        : "Pemilih file tidak dapat dibuka.";
    collection.notify("error", message);
  }
}

function edit(kind: CollectionDialog["kind"], item: MediaItem): void {
  dialog = { kind, item };
}
</script>

<svelte:head><title>LumaDesk — Koleksi</title></svelte:head>

<div class="app-shell">
  <header class="titlebar" data-tauri-drag-region>
    <div class="brand" data-tauri-drag-region>
      <span class="brand-mark"><Sparkles size={16} strokeWidth={2.4} /></span>
      <span data-tauri-drag-region>LumaDesk</span>
    </div>
    <span class="version" data-tauri-drag-region>v0.1.0</span>
  </header>

  <aside class="sidebar">
    <nav aria-label="Navigasi utama">
      {#each navigation as item}
        <button class:active={item.active} type="button" disabled={!item.active} aria-current={item.active ? "page" : undefined}>
          <item.icon size={18} />
          <span>{item.label}</span>
          {#if item.active}<small>{$collection.total}</small>{/if}
        </button>
      {/each}
    </nav>
    <div class="privacy-note">
      <span class="status-dot"></span>
      <div><strong>Lokal sepenuhnya</strong><small>Tanpa akun atau telemetri</small></div>
    </div>
  </aside>

  <div class:with-inspector={selectedItem !== undefined} class="workspace-region">
    <CollectionWorkspace
      state={$collection}
      onsearch={collection.setSearch}
      onkind={collection.setMediaKind}
      onfavoritefilter={collection.setFavoriteOnly}
      ontag={collection.setTag}
      onview={(view) => void collection.setView(view)}
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

{#if $collection.notice !== null}
  {#key $collection.notice.id}
    <Toast message={$collection.notice.message} tone={$collection.notice.tone} ondismiss={collection.dismissNotice} />
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

.titlebar {
  z-index: 10;
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--canvas-900);
  border-bottom: 1px solid var(--line-700);
  user-select: none;
}

.brand { display: flex; align-items: center; gap: 8px; font-family: "Space Grotesk Variable", sans-serif; font-weight: 680; }
.brand-mark { display: grid; width: 28px; height: 28px; place-items: center; color: var(--brand-300); background: var(--surface-800); border-radius: 6px; }
.version { color: var(--ink-500); font-size: 12px; }

.sidebar {
  display: flex;
  min-height: 0;
  flex-direction: column;
  justify-content: space-between;
  padding: 20px 12px 16px;
  background: var(--canvas-900);
  border-right: 1px solid var(--line-700);
}

nav { display: grid; gap: 4px; }
nav button {
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr) auto;
  width: 100%;
  min-height: 44px;
  align-items: center;
  gap: 12px;
  padding: 0 12px;
  color: var(--ink-300);
  text-align: left;
  background: transparent;
  border: 0;
  border-radius: 6px;
}
nav button.active { color: var(--ink-100); background: var(--surface-750); cursor: pointer; }
nav button:disabled:not(.active) { opacity: 0.52; }
nav small { color: var(--ink-500); font-size: 12px; }

.privacy-note { display: flex; align-items: flex-start; gap: 10px; padding: 12px; color: var(--ink-300); font-size: 12px; background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 8px; }
.privacy-note div { display: grid; gap: 3px; }
.privacy-note small { color: var(--ink-500); }
.status-dot { width: 8px; height: 8px; margin-top: 4px; background: var(--success-400); border-radius: 50%; }

.workspace-region { position: relative; display: grid; min-width: 0; min-height: 0; grid-template-columns: minmax(0, 1fr); }
.workspace-region.with-inspector { grid-template-columns: minmax(0, 1fr) 320px; }

@media (max-width: 1100px) {
  .workspace-region.with-inspector { grid-template-columns: minmax(0, 1fr); }
}

@media (max-width: 760px) {
  .app-shell { grid-template: 40px minmax(0, 1fr) 64px / 1fr; }
  .sidebar { z-index: 8; grid-row: 3; padding: 8px 12px; border-top: 1px solid var(--line-700); border-right: 0; }
  nav { grid-template-columns: repeat(3, 1fr); }
  nav button { display: flex; flex-direction: column; justify-content: center; gap: 3px; padding: 0 4px; font-size: 12px; text-align: center; }
  nav button small { display: none; }
  .privacy-note { display: none; }
  .workspace-region { grid-row: 2; }
}
</style>
