<script lang="ts">
import { Heart, MonitorUp, Pencil, X } from "@lucide/svelte";
import type { EditRecipe, MediaItem } from "../contracts";
import MediaInspectorDetails from "./MediaInspectorDetails.svelte";
import MediaPreview from "./MediaPreview.svelte";

export let item: MediaItem;
export let src: string | undefined = undefined;
export let recipe: EditRecipe | undefined = undefined;
export let busy = false;
export let onclose: () => void;
export let onrename: (item: MediaItem) => void;
export let ontags: (item: MediaItem) => void;
export let ondelete: (item: MediaItem) => void;
export let onfavorite: (item: MediaItem) => void;
export let onshow: (item: MediaItem) => void;
</script>

<aside class="inspector" aria-label={`${item.displayName} details`}>
  <header class="inspector__header">
    <div>
      <span class="eyebrow">ANIMATION DETAILS</span>
      <h2>{item.displayName}</h2>
    </div>
    <button class="icon-button" type="button" aria-label="Close details" onclick={onclose}>
      <X size={18} />
    </button>
  </header>

  <div class="inspector__body">
    <MediaPreview {item} {src} {recipe} eager />

    <button class="primary-button" type="button" disabled={busy} onclick={() => onshow(item)}>
      <MonitorUp size={17} />
      Show on desktop
    </button>

    <div class="inspector__actions">
      <button class="secondary-button" type="button" disabled={busy} onclick={() => onfavorite(item)}>
        <Heart size={16} fill={item.isFavorite ? "currentColor" : "none"} />
        {item.isFavorite ? "Favorited" : "Favorite"}
      </button>
      <button class="secondary-button" type="button" disabled={busy} onclick={() => onrename(item)}>
        <Pencil size={16} /> Rename
      </button>
    </div>

    <MediaInspectorDetails {item} {busy} {ontags} {ondelete} />
  </div>
</aside>

<style>
.inspector {
  z-index: 4;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-width: 0;
  min-height: 0;
  background: var(--canvas-900);
  border-left: 1px solid var(--line-700);
}

.inspector__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  min-width: 0;
  gap: 16px;
  padding: 20px;
  border-bottom: 1px solid var(--line-700);
}

.inspector__header > div {
  min-width: 0;
}

h2 {
  overflow: hidden;
  margin: 4px 0 0;
  font-size: 20px;
  line-height: 1.25;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.eyebrow {
  color: var(--brand-300);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
}

.inspector__body {
  min-height: 0;
  overflow: auto;
  padding: 20px;
}

.inspector__actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-top: 16px;
}

.icon-button,
.primary-button,
.secondary-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 40px;
  gap: 8px;
  color: var(--ink-300);
  font: inherit;
  background: var(--surface-800);
  border: 1px solid var(--line-700);
  border-radius: 6px;
  cursor: pointer;
  transition:
    filter 140ms ease,
    opacity 140ms ease,
    transform 140ms ease;
}

.primary-button {
  width: 100%;
  margin-top: 16px;
  color: var(--canvas-950);
  font-weight: 700;
  background: var(--brand-400);
  border-color: var(--brand-400);
}

.primary-button:hover {
  color: var(--canvas-950);
  background: var(--brand-300);
  border-color: var(--brand-300);
}

.icon-button {
  width: 40px;
  flex: 0 0 auto;
  background: transparent;
  border-color: transparent;
}

.icon-button:hover,
.secondary-button:hover {
  color: var(--ink-100);
  background: var(--surface-750);
}

.icon-button:hover,
.primary-button:hover,
.secondary-button:hover {
  filter: brightness(1.06);
}

.icon-button:active:not(:disabled),
.primary-button:active:not(:disabled),
.secondary-button:active:not(:disabled) {
  filter: brightness(0.96);
  transform: translateY(1px);
}

button:disabled {
  cursor: wait;
  opacity: 0.55;
}

@media (max-width: 1100px) {
  .inspector {
    position: absolute;
    inset-block: 0;
    inset-inline-end: 0;
    width: min(360px, 100%);
    box-shadow: 0 12px 36px color-mix(in srgb, var(--canvas-950) 70%, transparent);
  }
}

@media (max-width: 760px) {
  .inspector {
    inset-block: auto 0;
    width: 100%;
    max-height: 72%;
    border-top: 1px solid var(--line-700);
    border-left: 0;
  }
}
</style>
