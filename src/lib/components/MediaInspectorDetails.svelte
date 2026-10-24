<script lang="ts">
import { Tags, Trash2 } from "@lucide/svelte";
import type { MediaItem } from "../contracts";
import {
  mediaDate,
  mediaDimensions,
  mediaDuration,
  mediaFrameLabel,
  mediaKindLabel,
} from "../presentation";

export let item: MediaItem;
export let busy = false;
export let ontags: (item: MediaItem) => void;
export let ondelete: (item: MediaItem) => void;
</script>

<section class="inspector-section">
  <h3>Informasi</h3>
  <dl>
    <div><dt>Format</dt><dd>{mediaKindLabel(item.mediaKind)}</dd></div>
    <div><dt>Ukuran</dt><dd>{mediaDimensions(item)}</dd></div>
    <div><dt>Frame</dt><dd>{mediaFrameLabel(item.frameCount)}</dd></div>
    <div><dt>Durasi</dt><dd>{mediaDuration(item)}</dd></div>
    <div><dt>Ditambahkan</dt><dd>{mediaDate(item.createdAt)}</dd></div>
  </dl>
</section>

<section class="inspector-section">
  <div class="section-heading">
    <h3>Tag</h3>
    <button class="quiet-button" type="button" disabled={busy} onclick={() => ontags(item)}>
      <Tags size={15} /> Kelola
    </button>
  </div>
  {#if item.tags.length > 0}
    <div class="tag-cluster">
      {#each item.tags as tag}
        <span class="tag">{tag}</span>
      {/each}
    </div>
  {:else}
    <p class="muted-copy">Belum ada tag untuk animasi ini.</p>
  {/if}
</section>

<section class="inspector-section danger-zone">
  <h3>Kelola koleksi</h3>
  <p>Salinan terkelola dan cache akan dihapus. File sumber asli tetap aman.</p>
  <button class="danger-button" type="button" disabled={busy} onclick={() => ondelete(item)}>
    <Trash2 size={16} /> Hapus dari koleksi
  </button>
</section>

<style>
.inspector-section {
  padding-block: 20px;
  border-bottom: 1px solid var(--line-700);
}

.inspector-section:last-child {
  border-bottom: 0;
}

h3 {
  margin: 0 0 12px;
  font-size: 13px;
}

dl {
  display: grid;
  gap: 8px;
  margin: 0;
}

dl div {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  font-size: 13px;
}

dt {
  color: var(--ink-500);
}

dd {
  margin: 0;
  color: var(--ink-300);
  text-align: right;
}

.section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.section-heading h3 {
  margin: 0;
}

.tag-cluster {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tag {
  max-width: 100%;
  overflow: hidden;
  padding: 4px 8px;
  color: var(--ink-300);
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
  background: var(--surface-800);
  border: 1px solid var(--line-700);
  border-radius: 999px;
}

.muted-copy,
.danger-zone p {
  margin: 0;
  color: var(--ink-500);
  font-size: 13px;
}

.danger-zone p {
  margin-bottom: 12px;
}

.quiet-button,
.danger-button {
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
}

.quiet-button {
  min-height: 32px;
  padding-inline: 8px;
  font-size: 12px;
  background: transparent;
  border-color: transparent;
}

.danger-button {
  width: 100%;
  color: var(--danger-400);
  background: color-mix(in srgb, var(--danger-400) 8%, var(--surface-850));
  border-color: color-mix(in srgb, var(--danger-400) 34%, var(--line-700));
}

.quiet-button:hover {
  color: var(--ink-100);
  background: var(--surface-750);
}

.danger-button:hover {
  background: color-mix(in srgb, var(--danger-400) 14%, var(--surface-850));
}

button:disabled {
  cursor: wait;
  opacity: 0.55;
}
</style>
