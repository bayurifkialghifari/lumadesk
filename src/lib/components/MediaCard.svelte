<script lang="ts">
import { Check, Heart } from "@lucide/svelte";
import type { MediaId, MediaItem } from "../contracts";
import { mediaDimensions, mediaKindLabel } from "../presentation";
import MediaPreview from "./MediaPreview.svelte";

export let item: MediaItem;
export let src: string | undefined = undefined;
export let selected = false;
export let busy = false;
export let onselect: (id: MediaId) => void;
export let onfavorite: (item: MediaItem) => void;
</script>

<article class:selected class="media-card">
  <button
    class="media-card__body"
    type="button"
    aria-pressed={selected}
    onclick={() => onselect(item.id)}
  >
    <MediaPreview {item} {src} />
    <span class="media-card__copy">
      <span class="media-card__name" title={item.displayName}>{item.displayName}</span>
      <span class="media-card__meta">{mediaKindLabel(item.mediaKind)} · {mediaDimensions(item)}</span>
    </span>
    {#if selected}
      <span class="selected-mark" aria-hidden="true"><Check size={14} strokeWidth={2.5} /></span>
    {/if}
  </button>
  <button
    class:favorite={item.isFavorite}
    class="favorite-button"
    type="button"
    aria-label={item.isFavorite ? `Hapus ${item.displayName} dari favorit` : `Favoritkan ${item.displayName}`}
    title={item.isFavorite ? "Hapus dari favorit" : "Tambahkan ke favorit"}
    disabled={busy}
    onclick={() => onfavorite(item)}
  >
    <Heart size={17} fill={item.isFavorite ? "currentColor" : "none"} />
  </button>
</article>

<style>
.media-card {
  position: relative;
  min-width: 0;
  overflow: hidden;
  background: var(--surface-850);
  border: 1px solid var(--line-700);
  border-radius: 8px;
  transition:
    background-color 140ms ease,
    transform 140ms ease,
    border-color 140ms ease;
}

.media-card:hover {
  background: var(--surface-800);
  transform: translateY(-1px);
}

.media-card.selected {
  background: var(--surface-750);
  border-color: color-mix(in srgb, var(--ink-300) 52%, var(--line-700));
}

.media-card__body {
  display: grid;
  width: 100%;
  gap: 12px;
  padding: 8px 8px 12px;
  color: inherit;
  text-align: left;
  background: transparent;
  border: 0;
  cursor: pointer;
}

.media-card__body:focus-visible {
  outline: 2px solid var(--brand-400);
  outline-offset: -2px;
}

.media-card__copy {
  display: grid;
  min-width: 0;
  gap: 4px;
  padding-inline: 4px 36px;
}

.media-card__name {
  overflow: hidden;
  font-size: 14px;
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.media-card__meta {
  overflow: hidden;
  color: var(--ink-500);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.favorite-button {
  position: absolute;
  right: 12px;
  bottom: 12px;
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

.favorite-button:hover,
.favorite-button.favorite {
  color: var(--brand-300);
  background: var(--surface-800);
}

.favorite-button:disabled {
  cursor: wait;
  opacity: 0.55;
}

.selected-mark {
  position: absolute;
  top: 16px;
  left: 16px;
  display: grid;
  width: 24px;
  height: 24px;
  place-items: center;
  color: var(--canvas-950);
  background: var(--brand-300);
  border-radius: 50%;
}

@media (prefers-reduced-motion: reduce) {
  .media-card {
    transition-duration: 0.01ms;
  }

  .media-card:hover {
    transform: none;
  }
}
</style>
