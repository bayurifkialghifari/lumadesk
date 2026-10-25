<script lang="ts">
import { Grid2X2, List, Search, Upload, X } from "@lucide/svelte";
import { onMount } from "svelte";
import type { CollectionFilters, MediaKindFilter } from "../collection-model";
import type { AppSettings } from "../contracts";
import CollectionFiltersControl from "./CollectionFilters.svelte";

export let filters: CollectionFilters;
export let tags: readonly string[];
export let view: AppSettings["collectionView"];
export let importing = false;
export let refreshing = false;
export let onsearch: (value: string) => void;
export let onkind: (value: MediaKindFilter) => void;
export let onfavorite: (value: boolean) => void;
export let ontag: (value: string) => void;
export let onview: (value: AppSettings["collectionView"]) => void;
export let onimport: () => void;

let shortcutLabel = "Ctrl K";

onMount(() => {
  if (/Mac|iPhone|iPad/.test(navigator.userAgent)) shortcutLabel = "⌘ K";
});
</script>

<section class="toolbar" aria-label="Collection tools">
  <label class="search-field">
    <Search size={17} aria-hidden="true" />
    <span class="sr-only">Search collection</span>
    <input
      id="collection-search"
      type="search"
      placeholder="Search names or tags"
      value={filters.search}
      oninput={(event) => onsearch(event.currentTarget.value)}
    />
    {#if filters.search !== ""}
      <button type="button" aria-label="Clear search" onclick={() => onsearch("")}>
        <X size={15} />
      </button>
    {:else}
      <kbd>{shortcutLabel}</kbd>
    {/if}
  </label>

  <div class="filter-slot"><CollectionFiltersControl {filters} {tags} {onkind} {onfavorite} {ontag} /></div>

  <div class="toolbar__end">
    <div class="view-switcher" aria-label="Collection view">
      <button class:active={view === "grid"} type="button" aria-label="Grid view" onclick={() => onview("grid")}>
        <Grid2X2 size={16} />
      </button>
      <button class:active={view === "list"} type="button" aria-label="List view" onclick={() => onview("list")}>
        <List size={17} />
      </button>
    </div>
    <button class="primary-button" type="button" disabled={importing} onclick={onimport}>
      <Upload size={16} />
      {importing ? "Importing…" : "Import animation"}
    </button>
  </div>
</section>

{#if refreshing}
  <div class="refresh-line" role="progressbar" aria-label="Refreshing collection"></div>
{/if}

<style>
.toolbar {
  position: relative;
  display: grid;
  grid-template-columns: minmax(220px, 1fr) auto auto;
  align-items: center;
  gap: 12px;
  padding: 16px 24px;
  background: var(--surface-850);
  border-block: 1px solid var(--line-700);
}

.search-field {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  min-height: 40px;
  align-items: center;
  gap: 8px;
  padding-inline: 12px 8px;
  color: var(--ink-500);
  background: var(--canvas-950);
  border: 1px solid var(--line-700);
  border-radius: 6px;
}

.search-field:focus-within {
  border-color: var(--brand-400);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--brand-400) 24%, transparent);
}

input {
  min-width: 0;
  color: var(--ink-100);
  background: transparent;
  border: 0;
  outline: 0;
}

input { width: 100%; }
input::placeholder { color: var(--ink-500); }

kbd {
  padding: 2px 6px;
  color: var(--ink-500);
  font: 12px/1.5 "Inter Variable", sans-serif;
  background: var(--surface-800);
  border: 1px solid var(--line-700);
  border-radius: 4px;
}

.search-field button, .view-switcher button {
  display: grid;
  width: 32px;
  height: 32px;
  place-items: center;
  color: var(--ink-500);
  background: transparent;
  border: 0;
  border-radius: 6px;
  cursor: pointer;
}

.toolbar__end, .view-switcher {
  display: flex;
  align-items: center;
  gap: 8px;
}

.primary-button {
  min-height: 40px;
  padding-inline: 12px;
  border-radius: 6px;
  cursor: pointer;
}

.primary-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-weight: 650;
}

.view-switcher button.active {
  color: var(--ink-100);
  background: var(--surface-750);
}

.view-switcher {
  gap: 0;
  padding: 3px;
  background: var(--canvas-950);
  border: 1px solid var(--line-700);
  border-radius: 8px;
}

.primary-button {
  color: var(--canvas-950);
  background: var(--brand-400);
  border: 0;
}

.primary-button:hover { background: var(--brand-300); }
button:disabled { cursor: wait; opacity: 0.6; }

.refresh-line {
  position: absolute;
  z-index: 5;
  inset-inline: 0;
  height: 2px;
  overflow: hidden;
  background: color-mix(in srgb, var(--brand-400) 20%, transparent);
}

.refresh-line::after {
  position: absolute;
  width: 35%;
  height: 100%;
  content: "";
  background: var(--brand-400);
  animation: refresh 1s ease-in-out infinite;
}

@keyframes refresh {
  from { transform: translateX(-100%); }
  to { transform: translateX(390%); }
}

@media (max-width: 980px) {
  .toolbar { grid-template-columns: minmax(0, 1fr) auto; }
  .filter-slot { grid-column: 1 / -1; order: 3; }
}

@media (max-width: 620px) {
  .toolbar { padding: 12px 16px; }
  .toolbar__end .primary-button { width: 40px; padding: 0; }
  .toolbar__end .primary-button :global(svg) { margin: 0; }
  .toolbar__end .primary-button { font-size: 0; }
  kbd { display: none; }
}

@media (prefers-reduced-motion: reduce) {
  .refresh-line::after { animation: none; width: 100%; }
}
</style>
