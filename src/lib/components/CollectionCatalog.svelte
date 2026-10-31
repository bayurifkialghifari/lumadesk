<script lang="ts">
import { ChevronLeft, ChevronRight, Images, RefreshCw, SearchX, Upload } from "@lucide/svelte";
import { COLLECTION_PAGE_SIZE, type CollectionState, hasActiveFilters } from "../collection-model";
import type { MediaId, MediaItem } from "../contracts";
import MediaCard from "./MediaCard.svelte";
import MediaListRow from "./MediaListRow.svelte";

export let state: CollectionState;
export let onselect: (id: MediaId) => void;
export let onfavorite: (item: MediaItem) => void;
export let onpage: (page: number) => void;
export let onreload: () => void;
export let onimport: () => void;

$: pageCount = Math.max(1, Math.ceil(state.total / COLLECTION_PAGE_SIZE));
$: filtered = hasActiveFilters(state.filters);
</script>

<section class="catalog" aria-busy={state.loading || state.refreshing}>
  {#if state.loading}
    <div class="media-grid skeleton-grid" aria-label="Loading collection">
      {#each Array(8) as _}
        <div class="skeleton-card"><span></span><i></i><i></i></div>
      {/each}
    </div>
  {:else if state.loadError !== null}
    <div class="state-panel">
      <span class="state-icon error"><RefreshCw size={24} /></span>
      <div><h2>Collection could not be loaded</h2><p>{state.loadError}</p></div>
      <button class="secondary-button" type="button" onclick={onreload}>Try again</button>
    </div>
  {:else if state.items.length === 0 && !filtered}
    <div class="state-panel empty">
      <span class="state-icon"><Images size={28} /></span>
      <div>
        <h2>Your collection is ready</h2>
        <p>Import GIF, APNG, WebP, PNG, or JPG files. Everything stays on your device.</p>
      </div>
      <button class="primary-button" type="button" onclick={onimport}><Upload size={16} /> Import animation</button>
    </div>
  {:else if state.items.length === 0}
    <div class="state-panel">
      <span class="state-icon"><SearchX size={26} /></span>
      <div><h2>No results</h2><p>Try another search or broaden the collection filters.</p></div>
    </div>
  {:else}
    {#if state.settings.collectionView === "grid"}
      <div class="media-grid">
        {#each state.items as item (item.id)}
          <MediaCard
            {item}
            src={state.assetUrls[item.id]}
            recipe={state.editRecipes[item.id]}
            selected={state.selectedId === item.id}
            busy={state.busyIds.has(item.id)}
            {onselect}
            {onfavorite}
          />
        {/each}
      </div>
    {:else}
      <div class="media-list">
        <div class="list-heading" aria-hidden="true">
          <span>Animation</span><span>Format</span><span>Size</span><span>Frames</span><span></span>
        </div>
        {#each state.items as item (item.id)}
          <MediaListRow
            {item}
            src={state.assetUrls[item.id]}
            recipe={state.editRecipes[item.id]}
            selected={state.selectedId === item.id}
            busy={state.busyIds.has(item.id)}
            {onselect}
            {onfavorite}
          />
        {/each}
      </div>
    {/if}

    {#if pageCount > 1}
      <nav class="pagination" aria-label="Collection pages">
        <button type="button" aria-label="Previous page" disabled={state.page === 0} onclick={() => onpage(state.page - 1)}>
          <ChevronLeft size={17} />
        </button>
        <span>Page {state.page + 1} of {pageCount}</span>
        <button type="button" aria-label="Next page" disabled={state.page + 1 >= pageCount} onclick={() => onpage(state.page + 1)}>
          <ChevronRight size={17} />
        </button>
      </nav>
    {/if}
  {/if}
</section>

<style>
.catalog {
  min-height: 0;
  padding: 24px;
}

.media-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(184px, 100%), 1fr));
  gap: 16px;
}

.media-list {
  overflow: hidden;
  border: 1px solid var(--line-700);
  border-radius: 8px;
}

.list-heading {
  display: grid;
  grid-template-columns: minmax(240px, 1fr) 64px 116px 72px 48px;
  gap: 16px;
  padding: 8px 16px 8px 96px;
  color: var(--ink-500);
  font-size: 12px;
  background: var(--canvas-900);
  border-bottom: 1px solid var(--line-700);
}

.state-panel {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  max-width: 760px;
  align-items: center;
  gap: 20px;
  margin: 56px auto;
  padding: 24px;
  background: var(--surface-850);
  border: 1px solid var(--line-700);
  border-radius: 12px;
}

.state-panel.empty {
  background: linear-gradient(135deg, color-mix(in srgb, var(--brand-400) 9%, var(--surface-850)), var(--surface-850));
}

.state-icon {
  display: grid;
  width: 56px;
  height: 56px;
  place-items: center;
  color: var(--brand-300);
  background: var(--surface-750);
  border-radius: 12px;
}

.state-icon.error { color: var(--danger-400); }
h2 { margin: 0 0 4px; font-size: 20px; }
p { margin: 0; color: var(--ink-300); font-size: 13px; }

.primary-button, .secondary-button, .pagination button {
  display: inline-flex;
  min-height: 40px;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding-inline: 16px;
  font-weight: 650;
  border-radius: 6px;
  cursor: pointer;
}

.primary-button { color: var(--canvas-950); background: var(--brand-400); border: 0; }
.primary-button:hover { background: var(--brand-300); }
.secondary-button, .pagination button { color: var(--ink-300); background: var(--surface-800); border: 1px solid var(--line-700); }
.secondary-button:hover, .pagination button:hover:not(:disabled) { color: var(--ink-100); background: var(--surface-750); }

.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-top: 24px;
  color: var(--ink-500);
  font-size: 13px;
}

.pagination button { width: 40px; padding: 0; }
.pagination button:disabled { cursor: default; opacity: 0.45; }

.skeleton-card {
  display: grid;
  gap: 12px;
  padding: 8px 8px 12px;
  background: var(--surface-850);
  border: 1px solid var(--line-700);
  border-radius: 8px;
}

.skeleton-card span, .skeleton-card i {
  display: block;
  background: linear-gradient(90deg, var(--surface-800), var(--surface-750), var(--surface-800));
  background-size: 240% 100%;
  animation: shimmer 1.4s ease infinite;
}

.skeleton-card span { aspect-ratio: 4 / 3; border-radius: 8px; }
.skeleton-card i { width: 72%; height: 12px; margin-inline: 4px; border-radius: 4px; }
.skeleton-card i:last-child { width: 48%; height: 8px; }

@keyframes shimmer {
  from { background-position: 100% 0; }
  to { background-position: -140% 0; }
}

@container workspace (max-width: 820px) {
  .list-heading { grid-template-columns: minmax(0, 1fr) 64px 48px; padding-left: 88px; }
  .list-heading span:nth-child(3), .list-heading span:nth-child(4) { display: none; }
}

@media (max-width: 620px) {
  .catalog { padding: 16px; }
  .state-panel { grid-template-columns: 1fr; margin-block: 24px; text-align: center; }
  .state-icon { margin-inline: auto; }
  .state-panel .primary-button, .state-panel .secondary-button { width: 100%; }
}

@media (prefers-reduced-motion: reduce) {
  .skeleton-card span, .skeleton-card i { animation: none; }
}
</style>
