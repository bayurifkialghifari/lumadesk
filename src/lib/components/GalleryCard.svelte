<script lang="ts">
import { Check, Download, ExternalLink, ImageOff, LoaderCircle, Pause, Play } from "@lucide/svelte";
import { onMount } from "svelte";
import type { GalleryItem } from "../contracts";
import { formatGalleryBytes } from "../gallery-model";
import { mediaFormatLabel } from "../presentation";

export let item: GalleryItem;
export let src: string | undefined = undefined;
export let installed = false;
export let busy = false;
export let nativeAvailable: boolean;
export let onadd: (item: GalleryItem) => void;
export let onopen: (url: string) => void;

let previewFailed = false;
let reducedMotion = false;
let previewPlaying = false;

$: if (src !== undefined) {
  previewFailed = false;
  previewPlaying = false;
}

onMount(() => {
  const preference = window.matchMedia("(prefers-reduced-motion: reduce)");
  const syncPreference = (): void => {
    reducedMotion = preference.matches;
    if (reducedMotion) previewPlaying = false;
  };
  syncPreference();
  preference.addEventListener("change", syncPreference);
  return () => preference.removeEventListener("change", syncPreference);
});
</script>

<article class="gallery-card">
  <div class="preview">
    {#if src === undefined || previewFailed}
      <span class="preview-missing"><ImageOff size={24} /><small>Preview unavailable</small></span>
    {:else if reducedMotion && !previewPlaying}
      <span class="preview-missing preview-paused">
        <Pause size={24} />
        <small>Animated preview paused</small>
        <button type="button" onclick={() => (previewPlaying = true)}><Play size={13} /> Play preview</button>
      </span>
    {:else}
      <img
        src={src}
        alt={`${item.title} animated preview`}
        loading="lazy"
        onerror={() => (previewFailed = true)}
      />
      {#if reducedMotion}
        <button class="preview-toggle" type="button" onclick={() => (previewPlaying = false)}><Pause size={13} /> Pause</button>
      {/if}
    {/if}
    <span class="format-badge">{mediaFormatLabel(item.download.mediaFormat)}</span>
  </div>

  <div class="card-copy">
    <div class="title-row">
      <div>
        <h2>{item.title}</h2>
        <p>by {item.author}</p>
      </div>
      <span class="size">{formatGalleryBytes(item.download.sizeBytes)}</span>
    </div>
    <p class="description">{item.description}</p>
    <div class="tags" aria-label="Tags">
      {#each item.tags as tag}
        <span>{tag}</span>
      {/each}
    </div>
  </div>

  <footer>
    <div class="links">
      <button type="button" onclick={() => onopen(item.sourceUrl)}>Source <ExternalLink size={12} /></button>
      <button type="button" onclick={() => onopen(item.licenseUrl)}>{item.licenseName} <ExternalLink size={12} /></button>
    </div>
    <button
      class="add-button"
      class:installed
      type="button"
      disabled={busy || installed || !nativeAvailable}
      title={nativeAvailable ? undefined : "Install the desktop app to add Gallery items"}
      onclick={() => onadd(item)}
    >
      {#if busy}
        <LoaderCircle class="spin" size={15} /> Adding…
      {:else if installed}
        <Check size={15} /> In collection
      {:else}
        <Download size={15} /> Add to collection
      {/if}
    </button>
  </footer>
</article>

<style>
.gallery-card {
  display: grid;
  min-width: 0;
  overflow: hidden;
  background: var(--surface-850);
  border: 1px solid var(--line-700);
  border-radius: 9px;
  transition: transform 140ms ease, border-color 140ms ease, background-color 140ms ease;
}

.gallery-card:hover {
  background: var(--surface-800);
  border-color: color-mix(in srgb, var(--brand-400) 40%, var(--line-700));
  transform: translateY(-2px);
}

.preview {
  position: relative;
  display: grid;
  aspect-ratio: 16 / 10;
  place-items: center;
  overflow: hidden;
  background:
    linear-gradient(45deg, color-mix(in srgb, var(--surface-750) 55%, transparent) 25%, transparent 25%) 0 0 / 18px 18px,
    linear-gradient(-45deg, color-mix(in srgb, var(--surface-750) 55%, transparent) 25%, transparent 25%) 0 9px / 18px 18px,
    var(--canvas-950);
  border-bottom: 1px solid var(--line-700);
}

.preview img { width: 100%; height: 100%; object-fit: contain; }
.preview-missing { display: grid; justify-items: center; gap: 7px; color: var(--ink-500); }
.preview-missing small { font-size: 11px; }
.preview-paused button, .preview-toggle { display: inline-flex; min-height: 30px; align-items: center; gap: 5px; padding: 0 9px; color: var(--ink-300); font-size: 10px; font-weight: 650; background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 5px; cursor: pointer; }
.preview-paused button:hover, .preview-toggle:hover { color: var(--ink-100); background: var(--surface-750); }
.preview-toggle { position: absolute; right: 10px; bottom: 10px; background: color-mix(in srgb, var(--canvas-950) 88%, transparent); backdrop-filter: blur(6px); }
.format-badge { position: absolute; top: 10px; right: 10px; padding: 4px 7px; color: var(--ink-100); font-size: 10px; font-weight: 750; background: color-mix(in srgb, var(--canvas-950) 82%, transparent); border: 1px solid color-mix(in srgb, var(--line-700) 72%, transparent); border-radius: 5px; backdrop-filter: blur(6px); }
.card-copy { display: grid; align-content: start; gap: 12px; padding: 16px; }
.title-row { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; }
h2 { margin: 0; font-family: "Space Grotesk Variable", sans-serif; font-size: 16px; }
.title-row p { margin: 3px 0 0; color: var(--ink-500); font-size: 11px; }
.size { flex: 0 0 auto; color: var(--ink-500); font-size: 10px; font-variant-numeric: tabular-nums; }
.description { min-height: 38px; margin: 0; color: var(--ink-300); font-size: 12px; line-height: 1.55; }
.tags { display: flex; flex-wrap: wrap; gap: 5px; }
.tags span { padding: 3px 7px; color: var(--brand-300); font-size: 10px; background: color-mix(in srgb, var(--brand-400) 9%, var(--surface-800)); border: 1px solid color-mix(in srgb, var(--brand-400) 25%, var(--line-700)); border-radius: 999px; }
footer { display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 11px 12px; border-top: 1px solid var(--line-700); }
.links { display: flex; min-width: 0; gap: 2px; }
.links button { display: inline-flex; align-items: center; gap: 3px; padding: 6px; color: var(--ink-500); font-size: 10px; background: transparent; border: 0; border-radius: 5px; cursor: pointer; }
.links button:hover { color: var(--ink-100); background: var(--surface-750); }
.add-button { display: inline-flex; min-height: 34px; flex: 0 0 auto; align-items: center; gap: 6px; padding: 0 10px; color: var(--canvas-950); font-size: 11px; font-weight: 750; background: var(--brand-400); border: 1px solid var(--brand-400); border-radius: 6px; cursor: pointer; }
.add-button:hover { background: var(--brand-300); }
.add-button:focus-visible { outline-color: var(--ink-100); box-shadow: 0 0 0 2px var(--canvas-950); }
.add-button.installed { color: var(--success-400); background: color-mix(in srgb, var(--success-400) 10%, var(--surface-800)); border-color: color-mix(in srgb, var(--success-400) 35%, var(--line-700)); }
.add-button:disabled { cursor: default; opacity: .68; }
:global(.spin) { animation: gallery-spin 900ms linear infinite; }
@keyframes gallery-spin { to { transform: rotate(360deg); } }
@media (max-width: 460px) { footer { align-items: stretch; flex-direction: column; } .add-button { justify-content: center; } }
@media (prefers-reduced-motion: reduce) { .gallery-card { transition-duration: .01ms; } .gallery-card:hover { transform: none; } :global(.spin) { animation: none; } }
</style>
