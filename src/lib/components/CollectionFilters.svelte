<script lang="ts">
import { Heart } from "@lucide/svelte";
import { ALL_MEDIA_KINDS, type CollectionFilters, type MediaKindFilter } from "../collection-model";

export let filters: CollectionFilters;
export let tags: readonly string[];
export let onkind: (value: MediaKindFilter) => void;
export let onfavorite: (value: boolean) => void;
export let ontag: (value: string) => void;

function handleMediaKind(event: Event): void {
  if (!(event.currentTarget instanceof HTMLSelectElement)) return;
  const value = event.currentTarget.value;
  if (
    value === ALL_MEDIA_KINDS ||
    value === "gif" ||
    value === "apng" ||
    value === "webp" ||
    value === "png" ||
    value === "jpeg"
  ) {
    onkind(value);
  }
}
</script>

<div class="filter-group">
  <label>
    <span class="sr-only">Filter format</span>
    <select value={filters.mediaKind} onchange={handleMediaKind}>
      <option value={ALL_MEDIA_KINDS}>Semua format</option>
      <option value="gif">GIF</option>
      <option value="apng">APNG</option>
      <option value="webp">WebP</option>
      <option value="png">PNG</option>
      <option value="jpeg">JPG</option>
    </select>
  </label>

  <label>
    <span class="sr-only">Filter tag</span>
    <select value={filters.tag} onchange={(event) => ontag(event.currentTarget.value)}>
      <option value="">Semua tag</option>
      {#each tags as tag}
        <option value={tag}>{tag}</option>
      {/each}
    </select>
  </label>

  <button
    class:active={filters.favoriteOnly}
    type="button"
    aria-pressed={filters.favoriteOnly}
    onclick={() => onfavorite(!filters.favoriteOnly)}
  >
    <Heart size={16} fill={filters.favoriteOnly ? "currentColor" : "none"} />
    Favorit
  </button>
</div>

<style>
.filter-group { display: flex; width: 100%; align-items: center; gap: 8px; }
select, button { min-height: 40px; padding-inline: 12px; color: var(--ink-300); background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 6px; cursor: pointer; }
button { display: inline-flex; align-items: center; justify-content: center; gap: 8px; font-weight: 650; }
button.active { color: var(--ink-100); background: var(--surface-750); }
select:focus-visible, button:focus-visible { outline: 2px solid var(--brand-400); outline-offset: 2px; }

@media (max-width: 980px) {
  label { flex: 1; }
  select { width: 100%; }
}

@media (max-width: 620px) {
  .filter-group { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); }
  label { min-width: 0; }
  button { grid-column: 1 / -1; }
}
</style>
