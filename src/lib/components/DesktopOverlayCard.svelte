<script lang="ts">
import {
  Copy,
  Eye,
  EyeOff,
  FlipHorizontal2,
  Lock,
  Monitor,
  Pause,
  Play,
  Unlock,
  X,
} from "@lucide/svelte";
import type {
  DesktopOverlay,
  EditRecipe,
  MonitorDescriptor,
  OverlayId,
  OverlayPatch,
} from "../contracts";
import {
  MAX_OVERLAY_WIDTH,
  MIN_OVERLAY_WIDTH,
  normalizeOverlaySpeed,
  normalizeOverlayWidth,
  opacityFromPercent,
  overlayHeight,
} from "../desktop-model";
import { mediaFormatLabel } from "../presentation";
import MediaPreview from "./MediaPreview.svelte";

export let item: DesktopOverlay;
export let monitors: readonly MonitorDescriptor[];
export let src: string | undefined = undefined;
export let recipe: EditRecipe | undefined = undefined;
export let busy = false;
export let onupdate: (id: OverlayId, patch: OverlayPatch) => void;
export let onmovetomonitor: (id: OverlayId, monitorName: string) => void;
export let onduplicate: (id: OverlayId) => void;
export let onclose: (id: OverlayId) => void;

$: height = overlayHeight(item);
$: opacityPercent = Math.round(item.overlay.opacity * 100);
$: monitorAvailable = monitors.some((monitor) => monitor.name === item.overlay.monitorId);

function numberValue(event: Event): number {
  return Number((event.currentTarget as HTMLInputElement).value);
}

function selectValue(event: Event): string {
  return (event.currentTarget as HTMLSelectElement).value;
}
</script>

<article class:busy class:hidden={!item.overlay.isActive} class:locked={item.overlay.clickThrough} class="overlay-card">
  <div class="preview-column">
    <div
      class="preview-transform"
      style:opacity={item.overlay.opacity}
      style:transform={item.overlay.flipHorizontal ? "scaleX(-1)" : "none"}
    >
      <MediaPreview item={item.media} {src} {recipe} eager />
    </div>
    <div class="status-strip">
      <span class:off={!item.overlay.isActive}>{item.overlay.isActive ? "VISIBLE" : "HIDDEN"}</span>
      <span class:off={!item.overlay.isPlaying}>{item.overlay.isPlaying ? "PLAYING" : "PAUSED"}</span>
      {#if item.overlay.clickThrough}<span class="locked-status">LOCKED</span>{/if}
    </div>
  </div>

  <div class="overlay-copy">
    <div class="overlay-title">
      <div>
        <h2 title={item.media.displayName}>{item.media.displayName}</h2>
        <p><Monitor size={13} /> {item.overlay.monitorId ?? "Primary display"} · {Math.round(item.overlay.width)} × {height} px · {mediaFormatLabel(item.media.mediaFormat)}</p>
      </div>
      <div class="quick-actions">
        <button
          class:active={item.overlay.isActive}
          type="button"
          aria-label={item.overlay.isActive ? "Hide overlay" : "Show overlay"}
          title={item.overlay.isActive ? "Hide" : "Show"}
          disabled={busy}
          onclick={() => onupdate(item.overlay.id, { isActive: !item.overlay.isActive })}
        >
          {#if item.overlay.isActive}<EyeOff size={15} />{:else}<Eye size={15} />{/if}
        </button>
        <button
          class:active={item.overlay.isPlaying}
          type="button"
          aria-label={item.overlay.isPlaying ? "Pause overlay" : "Play overlay"}
          title={item.overlay.isPlaying ? "Pause" : "Play"}
          disabled={busy}
          onclick={() => onupdate(item.overlay.id, { isPlaying: !item.overlay.isPlaying })}
        >
          {#if item.overlay.isPlaying}<Pause size={15} />{:else}<Play size={15} />{/if}
        </button>
        <button
          class:active={item.overlay.flipHorizontal}
          type="button"
          aria-label="Flip overlay horizontally"
          title="Flip horizontally"
          disabled={busy}
          onclick={() => onupdate(item.overlay.id, { flipHorizontal: !item.overlay.flipHorizontal })}
        >
          <FlipHorizontal2 size={15} />
        </button>
        <button
          class:active={item.overlay.clickThrough}
          type="button"
          aria-label={item.overlay.clickThrough ? "Unlock overlay interaction" : "Make overlay click-through"}
          title={item.overlay.clickThrough ? "Unlock interaction" : "Lock as click-through"}
          disabled={busy}
          onclick={() => onupdate(item.overlay.id, { clickThrough: !item.overlay.clickThrough })}
        >
          {#if item.overlay.clickThrough}<Unlock size={15} />{:else}<Lock size={15} />{/if}
        </button>
        <button type="button" aria-label="Duplicate overlay" title="Duplicate" disabled={busy} onclick={() => onduplicate(item.overlay.id)}><Copy size={15} /></button>
        <button class="danger" type="button" aria-label="Close overlay" title="Close" disabled={busy} onclick={() => onclose(item.overlay.id)}><X size={15} /></button>
      </div>
    </div>

    <div class="control-grid">
      <label class="monitor-control">
        <span><strong>Display</strong></span>
        <select
          value={item.overlay.monitorId ?? monitors.find((monitor) => monitor.isPrimary)?.name ?? ""}
          disabled={busy || monitors.length === 0}
          aria-label={item.media.displayName + " display"}
          onchange={(event) => onmovetomonitor(item.overlay.id, selectValue(event))}
        >
          {#if item.overlay.monitorId !== null && !monitorAvailable}
            <option value={item.overlay.monitorId}>{item.overlay.monitorId} (unavailable)</option>
          {/if}
          {#each monitors as monitor (monitor.name)}
            <option value={monitor.name}>{monitor.name}{monitor.isPrimary ? " · Primary" : ""}</option>
          {/each}
        </select>
      </label>
      <label>
        <span><strong>Size</strong><output>{Math.round(item.overlay.width)} px</output></span>
        <input
          type="range"
          min={MIN_OVERLAY_WIDTH}
          max={MAX_OVERLAY_WIDTH}
          step="16"
          value={item.overlay.width}
          disabled={busy}
          aria-label={item.media.displayName + " width"}
          onchange={(event) => onupdate(item.overlay.id, { width: normalizeOverlayWidth(numberValue(event)) })}
        />
      </label>
      <label>
        <span><strong>Opacity</strong><output>{opacityPercent}%</output></span>
        <input
          type="range"
          min="10"
          max="100"
          step="5"
          value={opacityPercent}
          disabled={busy}
          aria-label={item.media.displayName + " opacity"}
          onchange={(event) => onupdate(item.overlay.id, { opacity: opacityFromPercent(numberValue(event)) })}
        />
      </label>
      <label>
        <span><strong>Speed</strong><output>{item.overlay.speed.toLocaleString("en-US")}×</output></span>
        <input
          type="range"
          min="0.25"
          max="4"
          step="0.25"
          value={item.overlay.speed}
          disabled={busy}
          aria-label={item.media.displayName + " speed"}
          onchange={(event) => onupdate(item.overlay.id, { speed: normalizeOverlaySpeed(numberValue(event)) })}
        />
      </label>
    </div>

    <div class="effect-row">
      <label class="effect-toggle">
        <input
          type="checkbox"
          checked={item.overlay.removeBackground}
          disabled={busy}
          onchange={() => onupdate(item.overlay.id, { removeBackground: !item.overlay.removeBackground })}
        />
        <span><strong>Quick edge removal</strong><small>Mask a solid background at playback time.</small></span>
      </label>
      <label class="effect-toggle">
        <input
          type="checkbox"
          checked={item.overlay.clickThrough}
          disabled={busy}
          onchange={() => onupdate(item.overlay.id, { clickThrough: !item.overlay.clickThrough })}
        />
        <span><strong>Click-through</strong><small>Mouse input passes to windows below. Unlock it here or from the tray.</small></span>
      </label>
    </div>
  </div>
</article>

<style>
.overlay-card { display: grid; grid-template-columns: 164px minmax(0, 1fr); min-width: 0; overflow: hidden; background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 7px; transition: border-color 140ms ease, opacity 140ms ease; }
.overlay-card:hover { border-color: color-mix(in srgb, var(--ink-500) 64%, var(--line-700)); }
.overlay-card.busy { opacity: 0.7; }
.overlay-card.hidden { border-style: dashed; }
.overlay-card.locked { border-color: color-mix(in srgb, var(--brand-400) 48%, var(--line-700)); }
.preview-column { padding: 10px; background: color-mix(in srgb, var(--canvas-900) 72%, var(--surface-850)); border-right: 1px solid var(--line-700); }
.preview-transform { transform-origin: center; }
.hidden .preview-transform { filter: saturate(0.45); opacity: 0.42 !important; }
.status-strip { display: flex; align-items: center; gap: 5px; margin-top: 8px; }
.status-strip span { padding: 3px 5px; color: var(--success-400); font-size: 8px; font-weight: 800; letter-spacing: 0.06em; background: color-mix(in srgb, var(--success-400) 10%, transparent); border: 1px solid color-mix(in srgb, var(--success-400) 28%, transparent); border-radius: 3px; }
.status-strip span.off { color: var(--warning-400); background: color-mix(in srgb, var(--warning-400) 9%, transparent); border-color: color-mix(in srgb, var(--warning-400) 28%, transparent); }
.status-strip span.locked-status { color: var(--brand-300); background: color-mix(in srgb, var(--brand-400) 10%, transparent); border-color: color-mix(in srgb, var(--brand-400) 34%, transparent); }
.overlay-copy { min-width: 0; padding: 13px 14px; }
.overlay-title { display: flex; align-items: flex-start; justify-content: space-between; gap: 14px; }
.overlay-title > div:first-child { min-width: 0; }
h2 { overflow: hidden; margin: 0; font-size: 14px; text-overflow: ellipsis; white-space: nowrap; }
p { display: flex; align-items: center; gap: 5px; margin: 4px 0 0; overflow: hidden; color: var(--ink-500); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.quick-actions { display: flex; flex: 0 0 auto; gap: 3px; }
.quick-actions button { display: grid; width: 31px; height: 31px; padding: 0; place-items: center; color: var(--ink-300); background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 4px; cursor: pointer; }
.quick-actions button:hover, .quick-actions button.active { color: var(--brand-300); background: var(--surface-750); }
.quick-actions button.danger:hover { color: var(--danger-400); }
.quick-actions button:disabled { cursor: wait; opacity: 0.5; }
.control-grid { display: grid; grid-template-columns: minmax(150px, 1.2fr) repeat(3, minmax(105px, 1fr)); gap: 13px; margin-top: 15px; }
label { display: grid; min-width: 0; gap: 8px; }
label > span { display: flex; align-items: center; justify-content: space-between; gap: 8px; color: var(--ink-500); font-size: 10px; }
label strong { color: var(--ink-300); font-weight: 650; }
output { font-variant-numeric: tabular-nums; }
select { width: 100%; height: 30px; padding: 0 8px; overflow: hidden; color: var(--ink-300); font-size: 10px; text-overflow: ellipsis; background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 4px; }
input[type="range"] { width: 100%; accent-color: var(--brand-400); cursor: pointer; }
input[type="range"]:disabled, select:disabled { cursor: wait; opacity: 0.55; }
.effect-row { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; margin-top: 13px; padding-top: 12px; border-top: 1px solid var(--line-700); }
.effect-toggle { display: flex; min-height: 34px; align-items: center; gap: 9px; cursor: pointer; }
.effect-toggle > input { flex: 0 0 auto; width: 16px; height: 16px; margin: 0; accent-color: var(--brand-400); }
.effect-toggle > input:disabled { cursor: wait; opacity: 0.55; }
.effect-toggle > span { display: grid; justify-content: start; gap: 1px; color: var(--ink-500); font-size: 9px; }
.effect-toggle strong { color: var(--ink-300); font-size: 10px; }
.effect-toggle small { font-size: 9px; line-height: 1.35; }

@container desktop-workspace (max-width: 840px) {
  .overlay-card { grid-template-columns: 132px minmax(0, 1fr); }
  .control-grid { grid-template-columns: repeat(2, minmax(120px, 1fr)); }
  .overlay-title { display: grid; }
}

@media (max-width: 620px) {
  .overlay-card { grid-template-columns: 1fr; }
  .preview-column { border-right: 0; border-bottom: 1px solid var(--line-700); }
  .overlay-title { display: grid; }
  .control-grid, .effect-row { grid-template-columns: 1fr; }
}
</style>
