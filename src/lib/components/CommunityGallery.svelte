<script lang="ts">
import { CloudOff, Link2, LoaderCircle, RefreshCw, Search, X } from "@lucide/svelte";
import { openUrl } from "@tauri-apps/plugin-opener";
import { onMount } from "svelte";
import {
  cacheGalleryPreview,
  fetchGalleryCatalog,
  importGalleryItem,
  importMediaUrl,
} from "../api";
import type { ToastTone } from "../collection-model";
import type { GalleryCatalogResponse, GalleryItem, ImportOutcome } from "../contracts";
import {
  ALL_GALLERY_FORMATS,
  availableGalleryFormats,
  availableGalleryTags,
  browserGalleryCatalog,
  browserGalleryPreviewUrls,
  filterGalleryItems,
  type GalleryFormatFilter,
} from "../gallery-model";
import { mediaFormatLabel, userErrorMessage } from "../presentation";
import GalleryCard from "./GalleryCard.svelte";
import Modal from "./Modal.svelte";

export let nativeAvailable: boolean;
export let onnotify: (tone: ToastTone, message: string) => void;
export let onimported: (outcome: ImportOutcome) => Promise<void>;

let catalog: GalleryCatalogResponse = browserGalleryCatalog();
let previewUrls: Readonly<Record<string, string>> = browserGalleryPreviewUrls();
let loading = nativeAvailable;
let refreshing = false;
let loadError: string | null = null;
let search = "";
let tag = "";
let mediaFormat: GalleryFormatFilter = ALL_GALLERY_FORMATS;
let busyIds = new Set<string>();
let urlDialogOpen = false;
let urlValue = "";
let urlName = "";
let urlBusy = false;

$: tags = availableGalleryTags(catalog.items);
$: formats = availableGalleryFormats(catalog.items);
$: filteredItems = filterGalleryItems(catalog.items, { search, tag, mediaFormat });
$: installedHashes = new Set(catalog.installedHashes);
$: sourceLabel =
  catalog.source === "network"
    ? "Live catalog"
    : catalog.source === "cache"
      ? "Cached catalog"
      : "Offline starter pack";

onMount(() => {
  if (nativeAvailable) void loadCatalog(false);
});

async function loadCatalog(refresh: boolean): Promise<void> {
  if (!nativeAvailable || refreshing) return;
  refreshing = refresh;
  loading = !refresh;
  loadError = null;
  try {
    const nextCatalog = await fetchGalleryCatalog(refresh);
    catalog = nextCatalog;
    await loadNativePreviews(nextCatalog.items);
  } catch (error) {
    loadError = userErrorMessage(error, "The Community Gallery could not be loaded.");
  } finally {
    loading = false;
    refreshing = false;
  }
}

async function loadNativePreviews(items: readonly GalleryItem[]): Promise<void> {
  const results = await Promise.allSettled(
    items.map(async (item) => ({ itemId: item.id, preview: await cacheGalleryPreview(item.id) })),
  );
  const next = { ...browserGalleryPreviewUrls() };
  for (const result of results) {
    if (result.status === "fulfilled") {
      next[result.value.itemId] = result.value.preview.path;
    }
  }
  previewUrls = next;
}

async function addItem(item: GalleryItem): Promise<void> {
  if (!nativeAvailable || busyIds.has(item.id)) return;
  busyIds = new Set([...busyIds, item.id]);
  try {
    const outcome = await importGalleryItem(item.id);
    if (!catalog.installedHashes.includes(item.download.sha256)) {
      catalog = {
        ...catalog,
        installedHashes: [...catalog.installedHashes, item.download.sha256],
      };
    }
    await onimported(outcome);
    onnotify(
      outcome.status === "imported" ? "success" : "info",
      outcome.status === "imported"
        ? `${item.title} was added to your collection.`
        : `${item.title} is already in your collection.`,
    );
  } catch (error) {
    onnotify("error", userErrorMessage(error, `${item.title} could not be imported.`));
  } finally {
    const next = new Set(busyIds);
    next.delete(item.id);
    busyIds = next;
  }
}

async function importUrl(): Promise<void> {
  if (!nativeAvailable || urlBusy || urlValue.trim() === "") return;
  urlBusy = true;
  try {
    const outcome = await importMediaUrl(urlValue.trim(), urlName.trim());
    await onimported(outcome);
    onnotify(
      outcome.status === "imported" ? "success" : "info",
      outcome.status === "imported"
        ? `${outcome.item.displayName} was added to your collection.`
        : `${outcome.item.displayName} is already in your collection.`,
    );
    urlDialogOpen = false;
    urlValue = "";
    urlName = "";
  } catch (error) {
    onnotify("error", userErrorMessage(error, "The media URL could not be imported."));
  } finally {
    urlBusy = false;
  }
}

async function openExternal(url: string): Promise<void> {
  try {
    if (nativeAvailable) await openUrl(url);
    else window.open(url, "_blank", "noopener,noreferrer");
  } catch (error) {
    onnotify("error", userErrorMessage(error, "The link could not be opened."));
  }
}

function showUrlDialog(): void {
  urlDialogOpen = true;
}
</script>

<main class="gallery-workspace">
  <header class="workspace-header">
    <div>
      <span class="eyebrow">COMMUNITY CATALOG</span>
      <h1>Community Gallery</h1>
      <p>Browse verified animation packs or import media from an HTTPS link.</p>
    </div>
    <div class="header-actions">
      <span class="catalog-source">{sourceLabel} · {catalog.items.length} items</span>
      <button class="secondary-button" type="button" onclick={showUrlDialog}><Link2 size={16} /> Import URL</button>
      <button class="icon-button" type="button" aria-label="Refresh Gallery" title="Refresh Gallery" disabled={!nativeAvailable || refreshing} onclick={() => void loadCatalog(true)}>
        <RefreshCw class={refreshing ? "spin" : undefined} size={16} />
      </button>
    </div>
  </header>

  {#if !nativeAvailable}
    <div class="browser-note" role="status"><CloudOff size={17} /><span><strong>Browser preview.</strong> Install the desktop app to add items to your collection.</span></div>
  {/if}

  <section class="gallery-toolbar" aria-label="Gallery filters">
    <label class="search-field">
      <Search size={17} aria-hidden="true" />
      <span class="sr-only">Search Gallery</span>
      <input type="search" placeholder="Search titles, authors, or tags" bind:value={search} />
      {#if search !== ""}<button type="button" aria-label="Clear search" onclick={() => (search = "")}><X size={15} /></button>{/if}
    </label>
    <label><span>Format</span><select bind:value={mediaFormat}><option value={ALL_GALLERY_FORMATS}>All formats</option>{#each formats as format}<option value={format}>{mediaFormatLabel(format)}</option>{/each}</select></label>
    <label><span>Tag</span><select bind:value={tag}><option value="">All tags</option>{#each tags as option}<option value={option}>{option}</option>{/each}</select></label>
  </section>

  {#if loadError !== null}
    <section class="state-panel error-state" role="alert"><strong>Gallery unavailable</strong><p>{loadError}</p><button type="button" onclick={() => void loadCatalog(true)}>Try again</button></section>
  {:else if loading}
    <section class="state-panel" role="status"><LoaderCircle class="spin" size={24} /><strong>Loading Gallery</strong><p>Checking the catalog and verified previews.</p></section>
  {:else if filteredItems.length === 0}
    <section class="state-panel"><strong>No matching items</strong><p>Clear a filter or try another search.</p><button type="button" onclick={() => { search = ""; tag = ""; mediaFormat = ALL_GALLERY_FORMATS; }}>Clear filters</button></section>
  {:else}
    <section class="gallery-grid" aria-label="Gallery items">
      {#each filteredItems as item (item.id)}
        <GalleryCard {item} src={previewUrls[item.id]} installed={installedHashes.has(item.download.sha256)} busy={busyIds.has(item.id)} {nativeAvailable} onadd={(value) => void addItem(value)} onopen={(url) => void openExternal(url)} />
      {/each}
    </section>
  {/if}
</main>

{#if urlDialogOpen}
  <Modal title="Import from URL" description="Download one GIF, APNG, WebP, PNG, or JPG over HTTPS." busy={urlBusy} onclose={() => (urlDialogOpen = false)}>
    <form class="url-form" onsubmit={(event) => { event.preventDefault(); void importUrl(); }}>
      <label><span>Media URL</span><input type="url" inputmode="url" required pattern="https://.*" placeholder="https://example.com/animation.gif" bind:value={urlValue} /></label>
      <label><span>Name <small>optional</small></span><input type="text" maxlength="120" placeholder="Uses the filename when empty" bind:value={urlName} /></label>
      <p>The file is copied into your managed collection. Downloads are limited to 100 MiB.</p>
      <div class="form-actions"><button type="button" disabled={urlBusy} onclick={() => (urlDialogOpen = false)}>Cancel</button><button class="primary-button" type="submit" disabled={!nativeAvailable || urlBusy || urlValue.trim() === ""} title={nativeAvailable ? undefined : "Install the desktop app to import media"}>{#if urlBusy}<LoaderCircle class="spin" size={15} /> Importing…{:else}<Link2 size={15} /> Import media{/if}</button></div>
    </form>
  </Modal>
{/if}

<style>
.gallery-workspace { min-width: 0; min-height: 0; overflow: auto; background: var(--canvas-950); }
.workspace-header { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; padding: 28px 24px 20px; background: radial-gradient(circle at 80% 0%, color-mix(in srgb, var(--brand-400) 13%, transparent), transparent 36%), var(--canvas-950); }
.eyebrow { color: var(--brand-300); font-size: 11px; font-weight: 750; letter-spacing: .09em; }
h1 { margin: 4px 0; font-family: "Space Grotesk Variable", sans-serif; font-size: 28px; }
.workspace-header p { margin: 0; color: var(--ink-300); font-size: 13px; }
.header-actions { display: flex; align-items: center; gap: 8px; }
.catalog-source { margin-right: 4px; color: var(--ink-500); font-size: 11px; }
.secondary-button, .icon-button { display: inline-flex; min-height: 40px; align-items: center; justify-content: center; gap: 7px; color: var(--ink-300); background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 6px; cursor: pointer; }
.secondary-button { padding: 0 12px; font-weight: 650; }
.icon-button { width: 40px; padding: 0; }
.secondary-button:hover, .icon-button:hover { color: var(--ink-100); background: var(--surface-750); }
.icon-button:disabled { cursor: default; opacity: .55; }
.browser-note { display: flex; align-items: center; gap: 10px; padding: 10px 24px; color: var(--ink-300); font-size: 12px; background: color-mix(in srgb, var(--warning-400) 8%, var(--surface-850)); border-top: 1px solid color-mix(in srgb, var(--warning-400) 28%, var(--line-700)); }
.browser-note :global(svg) { flex: 0 0 auto; color: var(--warning-400); }
.browser-note strong { color: var(--ink-100); }
.gallery-toolbar { display: grid; grid-template-columns: minmax(260px, 1fr) minmax(150px, auto) minmax(150px, auto); align-items: end; gap: 12px; padding: 16px 24px; background: var(--surface-850); border-block: 1px solid var(--line-700); }
.gallery-toolbar > label:not(.search-field) { display: grid; gap: 5px; color: var(--ink-500); font-size: 10px; font-weight: 700; letter-spacing: .04em; text-transform: uppercase; }
.search-field { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; min-height: 40px; align-items: center; gap: 8px; padding-inline: 12px 8px; color: var(--ink-500); background: var(--canvas-950); border: 1px solid var(--line-700); border-radius: 6px; }
.search-field:focus-within { border-color: var(--brand-400); box-shadow: 0 0 0 2px color-mix(in srgb, var(--brand-400) 24%, transparent); }
.search-field input { min-width: 0; color: var(--ink-100); background: transparent; border: 0; outline: 0; }
.search-field input::-webkit-search-cancel-button { appearance: none; }
.search-field input::placeholder { color: var(--ink-500); }
.search-field button { display: grid; width: 30px; height: 30px; place-items: center; color: var(--ink-500); background: transparent; border: 0; border-radius: 5px; cursor: pointer; }
select { min-width: 150px; height: 40px; padding: 0 30px 0 10px; color: var(--ink-100); background: var(--canvas-950); border: 1px solid var(--line-700); border-radius: 6px; }
.gallery-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 14px; padding: 24px; }
.state-panel { display: grid; min-height: 280px; place-content: center; justify-items: center; gap: 8px; padding: 32px; color: var(--ink-500); text-align: center; }
.state-panel strong { color: var(--ink-100); }
.state-panel p { max-width: 440px; margin: 0; font-size: 12px; }
.state-panel button { min-height: 36px; margin-top: 6px; padding: 0 12px; color: var(--ink-300); background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 6px; cursor: pointer; }
.error-state { color: var(--danger-400); }
.url-form { display: grid; gap: 16px; padding: 20px; }
.url-form label { display: grid; gap: 7px; color: var(--ink-300); font-size: 12px; font-weight: 650; }
.url-form label small { color: var(--ink-500); font-weight: 500; }
.url-form input { height: 40px; padding: 0 11px; color: var(--ink-100); background: var(--canvas-950); border: 1px solid var(--line-700); border-radius: 6px; }
.url-form > p { margin: 0; color: var(--ink-500); font-size: 11px; line-height: 1.5; }
.form-actions { display: flex; justify-content: flex-end; gap: 8px; padding-top: 4px; }
.form-actions button { display: inline-flex; min-height: 38px; align-items: center; gap: 7px; padding: 0 13px; color: var(--ink-300); background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 6px; cursor: pointer; }
.form-actions .primary-button { color: var(--canvas-950); font-weight: 750; background: var(--brand-400); border-color: var(--brand-400); }
.form-actions button:disabled { cursor: wait; opacity: .55; }
:global(.spin) { animation: gallery-spin 900ms linear infinite; }
@keyframes gallery-spin { to { transform: rotate(360deg); } }
@media (max-width: 900px) { .workspace-header { align-items: flex-start; flex-direction: column; } .header-actions { width: 100%; } .catalog-source { margin-right: auto; } }
@media (max-width: 680px) { .gallery-workspace { scrollbar-width: none; } .gallery-workspace::-webkit-scrollbar { display: none; } .workspace-header { padding: 20px 16px 16px; } .workspace-header p, .catalog-source { display: none; } .header-actions { justify-content: flex-end; } .gallery-toolbar { grid-template-columns: 1fr 1fr; padding: 12px 16px; } .search-field { grid-column: 1 / -1; } select { width: 100%; min-width: 0; } .gallery-grid { grid-template-columns: 1fr; padding: 16px; } .browser-note { padding-inline: 16px; } }
@media (prefers-reduced-motion: reduce) { :global(.spin) { animation: none; } }
</style>
