<script lang="ts">
import { Check, Heart } from "@lucide/svelte";
import type { MediaId, MediaItem } from "../contracts";
import { mediaDimensions, mediaFrameLabel, mediaKindLabel } from "../presentation";
import MediaPreview from "./MediaPreview.svelte";

export let item: MediaItem;
export let src: string | undefined = undefined;
export let selected = false;
export let busy = false;
export let onselect: (id: MediaId) => void;
export let onfavorite: (item: MediaItem) => void;
</script>

<article class:selected class="media-row">
  <button class="media-row__body" type="button" aria-pressed={selected} onclick={() => onselect(item.id)}>
    <span class="media-row__preview"><MediaPreview {item} {src} /></span>
    <span class="media-row__identity">
      <span class="media-row__name" title={item.displayName}>{item.displayName}</span>
      <span class="media-row__tags">{item.tags.length > 0 ? item.tags.join(", ") : "Belum ada tag"}</span>
    </span>
    <span class="media-row__cell">{mediaKindLabel(item.mediaKind)}</span>
    <span class="media-row__cell">{mediaDimensions(item)}</span>
    <span class="media-row__cell">{mediaFrameLabel(item.frameCount)}</span>
    {#if selected}
      <span class="selected-mark" aria-hidden="true"><Check size={13} strokeWidth={2.5} /></span>
    {/if}
  </button>
  <button
    class:favorite={item.isFavorite}
    class="favorite-button"
    type="button"
    aria-label={item.isFavorite ? `Hapus ${item.displayName} dari favorit` : `Favoritkan ${item.displayName}`}
    disabled={busy}
    onclick={() => onfavorite(item)}
  >
    <Heart size={17} fill={item.isFavorite ? "currentColor" : "none"} />
  </button>
</article>

<style>
.media-row {
  position: relative;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 48px;
  min-width: 0;
  background: var(--surface-850);
  border-bottom: 1px solid var(--line-700);
  transition: background-color 140ms ease;
}

.media-row:first-child {
  border-radius: 8px 8px 0 0;
}

.media-row:last-child {
  border-bottom: 0;
  border-radius: 0 0 8px 8px;
}

.media-row:only-child {
  border-radius: 8px;
}

.media-row:hover,
.media-row.selected {
  background: var(--surface-800);
}

.media-row.selected {
  background: var(--surface-750);
}

.media-row__body {
  display: grid;
  grid-template-columns: 72px minmax(160px, 1fr) 64px 116px 72px;
  min-width: 0;
  align-items: center;
  gap: 16px;
  padding: 8px 16px 8px 8px;
  color: inherit;
  text-align: left;
  background: transparent;
  border: 0;
  cursor: pointer;
}

.media-row__body:focus-visible {
  outline: 2px solid var(--brand-400);
  outline-offset: -2px;
}

.media-row__preview {
  display: block;
  min-width: 0;
}

.media-row__identity {
  display: grid;
  min-width: 0;
  gap: 4px;
}

.media-row__name,
.media-row__tags {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.media-row__name {
  font-size: 14px;
  font-weight: 650;
}

.media-row__tags,
.media-row__cell {
  color: var(--ink-500);
  font-size: 12px;
}

.favorite-button {
  display: grid;
  width: 40px;
  height: 40px;
  margin: auto;
  place-items: center;
  color: var(--ink-500);
  background: transparent;
  border: 0;
  border-radius: 6px;
  cursor: pointer;
}

.favorite-button:hover,
.favorite-button.favorite {
  color: var(--brand-300);
  background: var(--surface-750);
}

.favorite-button:disabled {
  cursor: wait;
  opacity: 0.55;
}

.selected-mark {
  position: absolute;
  top: 12px;
  left: 12px;
  display: grid;
  width: 20px;
  height: 20px;
  place-items: center;
  color: var(--canvas-950);
  background: var(--brand-300);
  border-radius: 50%;
}

@container workspace (max-width: 820px) {
  .media-row__body {
    grid-template-columns: 64px minmax(0, 1fr) 64px;
  }

  .media-row__cell:nth-of-type(n + 4) {
    display: none;
  }
}
</style>
