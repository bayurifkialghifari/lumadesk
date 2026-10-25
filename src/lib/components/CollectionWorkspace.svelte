<script lang="ts">
import { CloudOff, Download, RefreshCw } from "@lucide/svelte";
import { onMount } from "svelte";
import { availableTags, type CollectionState, type MediaKindFilter } from "../collection-model";
import type { AppSettings, MediaId, MediaItem } from "../contracts";
import CollectionCatalog from "./CollectionCatalog.svelte";
import CollectionToolbar from "./CollectionToolbar.svelte";

export let state: CollectionState;
export let onsearch: (value: string) => void;
export let onkind: (value: MediaKindFilter) => void;
export let onfavoritefilter: (value: boolean) => void;
export let ontag: (value: string) => void;
export let onview: (value: AppSettings["collectionView"]) => void;
export let onimport: () => void;
export let onselect: (id: MediaId) => void;
export let onfavorite: (item: MediaItem) => void;
export let onpage: (page: number) => void;
export let onreload: () => void;

$: tags = availableTags(state.items);

onMount(() => {
  function handleShortcut(event: KeyboardEvent): void {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      const search = document.getElementById("collection-search");
      if (search instanceof HTMLInputElement) {
        event.preventDefault();
        search.focus();
      }
    }
  }
  window.addEventListener("keydown", handleShortcut);
  return () => window.removeEventListener("keydown", handleShortcut);
});
</script>

<main class="workspace">
  <header class="workspace-header">
    <div>
      <span class="eyebrow">LOCAL LIBRARY</span>
      <h1>Collection</h1>
      <p>Organize animations, transparent images, and favorite loops in one place.</p>
    </div>
    <div class="collection-summary">
      <strong>{state.total}</strong>
      <span>saved items</span>
      <button type="button" aria-label="Reload collection" disabled={state.refreshing} onclick={onreload}>
        <RefreshCw size={16} />
      </button>
    </div>
  </header>

  {#if !state.nativeAvailable}
    <div class="browser-note" role="status">
      <CloudOff size={17} />
      <span><strong>Browser preview.</strong> The native catalog and file picker are available in the desktop app.</span>
    </div>
  {/if}

  <CollectionToolbar
    filters={state.filters}
    {tags}
    view={state.settings.collectionView}
    importing={state.importing}
    refreshing={state.refreshing}
    {onsearch}
    {onkind}
    onfavorite={onfavoritefilter}
    {ontag}
    {onview}
    {onimport}
  />

  <CollectionCatalog
    {state}
    {onselect}
    {onfavorite}
    {onpage}
    {onreload}
    {onimport}
  />

  {#if state.dragActive}
    <div class="drop-curtain" role="status">
      <span><Download size={28} /></span>
      <strong>Drop files to import</strong>
      <p>GIF, APNG, WebP, PNG, and JPG</p>
    </div>
  {/if}
</main>

<style>
.workspace {
  position: relative;
  container: workspace / inline-size;
  min-width: 0;
  min-height: 0;
  overflow: auto;
  background: var(--canvas-950);
}

.workspace-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 24px;
  padding: 28px 24px 20px;
  background:
    radial-gradient(circle at 72% 0%, color-mix(in srgb, var(--brand-400) 12%, transparent), transparent 36%),
    var(--canvas-950);
}

.eyebrow {
  color: var(--brand-300);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
}

h1 {
  margin: 4px 0;
  font-family: "Space Grotesk Variable", sans-serif;
  font-size: 28px;
  line-height: 1.2;
}

.workspace-header p { margin: 0; color: var(--ink-300); font-size: 13px; }

.collection-summary {
  display: grid;
  grid-template-columns: auto auto 40px;
  align-items: center;
  gap: 8px;
  color: var(--ink-500);
  font-size: 12px;
}

.collection-summary strong { color: var(--ink-100); font-size: 20px; }
.collection-summary button {
  display: grid;
  width: 40px;
  height: 40px;
  margin-left: 8px;
  place-items: center;
  color: var(--ink-300);
  background: var(--surface-850);
  border: 1px solid var(--line-700);
  border-radius: 6px;
  cursor: pointer;
}

.collection-summary button:hover { color: var(--ink-100); background: var(--surface-800); }
.collection-summary button:disabled { cursor: wait; opacity: 0.55; }

.browser-note {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 24px;
  color: var(--ink-300);
  font-size: 12px;
  background: color-mix(in srgb, var(--warning-400) 8%, var(--surface-850));
  border-top: 1px solid color-mix(in srgb, var(--warning-400) 28%, var(--line-700));
}

.browser-note :global(svg) { flex: 0 0 auto; color: var(--warning-400); }
.browser-note strong { color: var(--ink-100); }

.drop-curtain {
  position: absolute;
  z-index: 20;
  inset: 12px;
  display: grid;
  place-content: center;
  justify-items: center;
  color: var(--ink-100);
  text-align: center;
  background: color-mix(in srgb, var(--canvas-950) 88%, transparent);
  border: 1px dashed var(--brand-300);
  border-radius: 12px;
  backdrop-filter: blur(8px);
}

.drop-curtain span {
  display: grid;
  width: 64px;
  height: 64px;
  margin-bottom: 16px;
  place-items: center;
  color: var(--brand-300);
  background: var(--surface-750);
  border-radius: 12px;
}

.drop-curtain strong { font-size: 20px; }
.drop-curtain p { margin: 4px 0 0; color: var(--ink-300); font-size: 13px; }

@media (max-width: 620px) {
  .workspace-header { align-items: flex-start; padding: 20px 16px 16px; }
  .workspace-header p, .collection-summary span { display: none; }
  .collection-summary { grid-template-columns: auto 40px; }
  .browser-note { padding-inline: 16px; }
}
</style>
