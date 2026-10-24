<script lang="ts">
import { Copy, FlipHorizontal2, Monitor, Pause, Play, X } from "@lucide/svelte";
import type { DesktopOverlay, OverlayId, OverlayPatch } from "../contracts";
import {
  MAX_OVERLAY_WIDTH,
  MIN_OVERLAY_WIDTH,
  normalizeOverlaySpeed,
  normalizeOverlayWidth,
  opacityFromPercent,
  overlayHeight,
} from "../desktop-model";
import { mediaKindLabel } from "../presentation";
import MediaPreview from "./MediaPreview.svelte";

export let item: DesktopOverlay;
export let src: string | undefined = undefined;
export let busy = false;
export let onupdate: (id: OverlayId, patch: OverlayPatch) => void;
export let onduplicate: (id: OverlayId) => void;
export let onclose: (id: OverlayId) => void;

$: height = overlayHeight(item);
$: opacityPercent = Math.round(item.overlay.opacity * 100);

function numberValue(event: Event): number {
  return Number((event.currentTarget as HTMLInputElement).value);
}
</script>

<article class:busy class="overlay-card">
  <div class="preview-column">
    <div
      class="preview-transform"
      style:opacity={item.overlay.opacity}
      style:transform={item.overlay.flipHorizontal ? "scaleX(-1)" : "none"}
    >
      <MediaPreview item={item.media} {src} eager />
    </div>
    <div class="preview-status">
      <span class:paused={!item.overlay.isPlaying}>
        {#if item.overlay.isPlaying}<Play size={11} fill="currentColor" /> Aktif{:else}<Pause size={11} /> Dijeda{/if}
      </span>
      <small>{mediaKindLabel(item.media.mediaKind)}</small>
    </div>
  </div>

  <div class="overlay-copy">
    <div class="overlay-title">
      <div>
        <h2 title={item.media.displayName}>{item.media.displayName}</h2>
        <p><Monitor size={13} /> {item.overlay.monitorId ?? "Monitor utama"} · {Math.round(item.overlay.width)} × {height} px</p>
      </div>
      <div class="quick-actions">
        <button
          class:active={item.overlay.isPlaying}
          type="button"
          aria-label={item.overlay.isPlaying ? "Jeda overlay" : "Putar overlay"}
          title={item.overlay.isPlaying ? "Jeda" : "Putar"}
          disabled={busy}
          onclick={() => onupdate(item.overlay.id, { isPlaying: !item.overlay.isPlaying })}
        >
          {#if item.overlay.isPlaying}<Pause size={16} />{:else}<Play size={16} />{/if}
        </button>
        <button
          class:active={item.overlay.flipHorizontal}
          type="button"
          aria-label="Balik overlay secara horizontal"
          title="Balik horizontal"
          disabled={busy}
          onclick={() => onupdate(item.overlay.id, { flipHorizontal: !item.overlay.flipHorizontal })}
        >
          <FlipHorizontal2 size={16} />
        </button>
        <button type="button" aria-label="Duplikat overlay" title="Duplikat" disabled={busy} onclick={() => onduplicate(item.overlay.id)}>
          <Copy size={16} />
        </button>
        <button class="danger" type="button" aria-label="Tutup overlay" title="Tutup" disabled={busy} onclick={() => onclose(item.overlay.id)}>
          <X size={16} />
        </button>
      </div>
    </div>

    <div class="control-grid">
      <label>
        <span><strong>Ukuran</strong><output>{Math.round(item.overlay.width)} px</output></span>
        <input
          type="range"
          min={MIN_OVERLAY_WIDTH}
          max={MAX_OVERLAY_WIDTH}
          step="16"
          value={item.overlay.width}
          disabled={busy}
          aria-label={`Lebar ${item.media.displayName}`}
          onchange={(event) => onupdate(item.overlay.id, { width: normalizeOverlayWidth(numberValue(event)) })}
        />
      </label>
      <label>
        <span><strong>Opasitas</strong><output>{opacityPercent}%</output></span>
        <input
          type="range"
          min="10"
          max="100"
          step="5"
          value={opacityPercent}
          disabled={busy}
          aria-label={`Opasitas ${item.media.displayName}`}
          onchange={(event) => onupdate(item.overlay.id, { opacity: opacityFromPercent(numberValue(event)) })}
        />
      </label>
      <label>
        <span><strong>Kecepatan</strong><output>{item.overlay.speed.toLocaleString("id-ID")}×</output></span>
        <input
          type="range"
          min="0.25"
          max="4"
          step="0.25"
          value={item.overlay.speed}
          disabled={busy}
          aria-label={`Kecepatan ${item.media.displayName}`}
          onchange={(event) => onupdate(item.overlay.id, { speed: normalizeOverlaySpeed(numberValue(event)) })}
        />
      </label>
    </div>
  </div>
</article>

<style>
.overlay-card {
  display: grid;
  grid-template-columns: 184px minmax(0, 1fr);
  min-width: 0;
  overflow: hidden;
  background: var(--surface-850);
  border: 1px solid var(--line-700);
  border-radius: 8px;
  transition: border-color 140ms ease, opacity 140ms ease;
}

.overlay-card:hover { border-color: color-mix(in srgb, var(--ink-500) 64%, var(--line-700)); }
.overlay-card.busy { opacity: 0.7; }

.preview-column { padding: 12px; background: color-mix(in srgb, var(--canvas-900) 72%, var(--surface-850)); border-right: 1px solid var(--line-700); }
.preview-transform { transform-origin: center; }
.preview-status { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 10px; color: var(--ink-500); }
.preview-status span { display: inline-flex; align-items: center; gap: 5px; color: var(--success-400); font-size: 11px; font-weight: 700; }
.preview-status span.paused { color: var(--warning-400); }
.preview-status small { overflow: hidden; font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }

.overlay-copy { min-width: 0; padding: 16px; }
.overlay-title { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; }
.overlay-title > div:first-child { min-width: 0; }
h2 { overflow: hidden; margin: 0; font-size: 16px; text-overflow: ellipsis; white-space: nowrap; }
p { display: flex; align-items: center; gap: 5px; margin: 5px 0 0; color: var(--ink-500); font-size: 12px; }

.quick-actions { display: flex; flex: 0 0 auto; gap: 4px; }
.quick-actions button {
  display: grid;
  width: 36px;
  height: 36px;
  padding: 0;
  place-items: center;
  color: var(--ink-300);
  background: var(--surface-800);
  border: 1px solid var(--line-700);
  border-radius: 6px;
  cursor: pointer;
}
.quick-actions button:hover, .quick-actions button.active { color: var(--brand-300); background: var(--surface-750); }
.quick-actions button.danger:hover { color: var(--danger-400); }
.quick-actions button:disabled { cursor: wait; opacity: 0.5; }

.control-grid { display: grid; grid-template-columns: repeat(3, minmax(120px, 1fr)); gap: 16px; margin-top: 20px; }
label { display: grid; gap: 10px; min-width: 0; }
label > span { display: flex; align-items: center; justify-content: space-between; gap: 8px; color: var(--ink-500); font-size: 11px; }
label strong { color: var(--ink-300); font-weight: 650; }
output { font-variant-numeric: tabular-nums; }
input[type="range"] { width: 100%; accent-color: var(--brand-400); cursor: pointer; }
input[type="range"]:disabled { cursor: wait; opacity: 0.55; }

@container desktop-workspace (max-width: 760px) {
  .overlay-card { grid-template-columns: 132px minmax(0, 1fr); }
  .control-grid { grid-template-columns: 1fr; gap: 12px; }
  .overlay-title { display: grid; }
}

@media (max-width: 620px) {
  .overlay-card { grid-template-columns: 1fr; }
  .preview-column { border-right: 0; border-bottom: 1px solid var(--line-700); }
  .overlay-title { display: flex; }
}
</style>
