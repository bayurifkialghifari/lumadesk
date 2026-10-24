<script lang="ts">
import { Image as ImageIcon } from "@lucide/svelte";
import type { MediaItem } from "../contracts";

export let item: MediaItem;
export let src: string | undefined = undefined;
export let eager = false;

let failed = false;
$: if (src !== undefined) {
  failed = false;
}
</script>

<div class="media-preview">
  {#if src !== undefined && !failed}
    <img
      src={src}
      alt={`Pratinjau ${item.displayName}`}
      width={item.width}
      height={item.height}
      loading={eager ? "eager" : "lazy"}
      decoding="async"
      draggable="false"
      onerror={() => (failed = true)}
    />
  {:else}
    <div class="preview-placeholder" aria-label="Pratinjau belum tersedia">
      <ImageIcon size={24} strokeWidth={1.7} />
    </div>
  {/if}
</div>

<style>
.media-preview {
  display: grid;
  width: 100%;
  min-width: 0;
  overflow: hidden;
  aspect-ratio: 4 / 3;
  place-items: center;
  background-color: var(--canvas-950);
  background-image:
    linear-gradient(45deg, color-mix(in srgb, var(--ink-100) 5%, transparent) 25%, transparent 25%),
    linear-gradient(-45deg, color-mix(in srgb, var(--ink-100) 5%, transparent) 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, color-mix(in srgb, var(--ink-100) 5%, transparent) 75%),
    linear-gradient(-45deg, transparent 75%, color-mix(in srgb, var(--ink-100) 5%, transparent) 75%);
  background-position: 0 0, 0 8px, 8px -8px, -8px 0;
  background-size: 16px 16px;
  border-radius: 8px;
}

img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.preview-placeholder {
  display: grid;
  width: 100%;
  height: 100%;
  place-items: center;
  color: var(--ink-500);
}
</style>
