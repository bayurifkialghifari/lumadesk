<script lang="ts">
import {
  CloudOff,
  Eye,
  EyeOff,
  Layers3,
  Lock,
  Pause,
  Play,
  RefreshCw,
  Trash2,
  Unlock,
} from "@lucide/svelte";
import type { DesktopSceneId, OverlayId, OverlayPatch } from "../contracts";
import type { DesktopState } from "../desktop-model";
import DesktopOverlayCard from "./DesktopOverlayCard.svelte";
import DesktopSceneManager from "./DesktopSceneManager.svelte";

export let state: DesktopState;
export let onupdate: (id: OverlayId, patch: OverlayPatch) => void;
export let onmovetomonitor: (id: OverlayId, monitorName: string) => void;
export let onduplicate: (id: OverlayId) => void;
export let onclose: (id: OverlayId) => void;
export let onshowall: () => void;
export let onhideall: () => void;
export let onplayall: () => void;
export let onpauseall: () => void;
export let onlockall: () => void;
export let onunlockall: () => void;
export let oncloseall: () => void;
export let oncreatescene: (name: string) => void;
export let onapplyscene: (id: DesktopSceneId) => void;
export let onupdatescene: (id: DesktopSceneId) => void;
export let onrenamescene: (id: DesktopSceneId, name: string) => void;
export let onduplicatescene: (id: DesktopSceneId) => void;
export let ondeletescene: (id: DesktopSceneId) => void;
export let onreload: () => void;
export let oncollection: () => void;

let confirmingCloseAll = false;
$: visibleCount = state.overlays.filter((item) => item.overlay.isActive).length;
$: playingCount = state.overlays.filter((item) => item.overlay.isPlaying).length;
$: lockedCount = state.overlays.filter((item) => item.overlay.clickThrough).length;
</script>

<main class="desktop-workspace">
  <header class="workspace-header">
    <div>
      <span class="eyebrow">DESKTOP CONTROL</span>
      <h1>Overlay Console</h1>
      <p>Build layouts, route overlays to displays, and control every instance from one place.</p>
    </div>
    <div class="desktop-summary" aria-label="Overlay status summary">
      <span><strong>{state.overlays.length}</strong> total</span>
      <i></i>
      <span><strong>{visibleCount}</strong> visible</span>
      <i></i>
      <span><strong>{playingCount}</strong> playing</span>
      <i></i>
      <span><strong>{lockedCount}</strong> locked</span>
      <button type="button" aria-label="Reload desktop data" title="Reload" disabled={state.refreshing} onclick={onreload}>
        <RefreshCw size={15} />
      </button>
    </div>
  </header>

  {#if !state.nativeAvailable}
    <div class="browser-note" role="status">
      <CloudOff size={17} />
      <span><strong>Browser preview.</strong> Native overlay windows are available in the desktop app.</span>
    </div>
  {/if}

  <section class="action-bar" aria-label="Global desktop overlay controls">
    <span class="bar-label">GLOBAL</span>
    <div class="action-group">
      <button type="button" disabled={state.batchBusy || visibleCount === state.overlays.length} onclick={onshowall}><Eye size={15} /> Show all</button>
      <button type="button" disabled={state.batchBusy || visibleCount === 0} onclick={onhideall}><EyeOff size={15} /> Hide all</button>
    </div>
    <div class="action-group">
      <button type="button" disabled={state.batchBusy || playingCount === state.overlays.length} onclick={onplayall}><Play size={15} /> Play all</button>
      <button type="button" disabled={state.batchBusy || playingCount === 0} onclick={onpauseall}><Pause size={15} /> Pause all</button>
    </div>
    <div class="action-group">
      <button type="button" disabled={state.batchBusy || lockedCount === state.overlays.length} onclick={onlockall}><Lock size={15} /> Lock all</button>
      <button type="button" disabled={state.batchBusy || lockedCount === 0} onclick={onunlockall}><Unlock size={15} /> Unlock all</button>
    </div>
    <div class="danger-group">
      {#if confirmingCloseAll}
        <span class="close-confirm">
          <small>Delete {state.overlays.length} overlay {state.overlays.length === 1 ? "instance" : "instances"}?</small>
          <button class="danger-confirm" type="button" disabled={state.batchBusy} onclick={() => { confirmingCloseAll = false; oncloseall(); }}>Close all</button>
          <button type="button" onclick={() => (confirmingCloseAll = false)}>Cancel</button>
        </span>
      {:else}
        <button class="danger-action" type="button" disabled={state.batchBusy || state.overlays.length === 0} onclick={() => (confirmingCloseAll = true)}>
          <Trash2 size={15} /> Close all
        </button>
      {/if}
    </div>
  </section>

  <DesktopSceneManager
    scenes={state.scenes}
    overlayCount={state.overlays.length}
    busy={state.sceneBusy || state.batchBusy}
    oncreate={oncreatescene}
    onapply={onapplyscene}
    onupdate={onupdatescene}
    onrename={onrenamescene}
    onduplicate={onduplicatescene}
    ondelete={ondeletescene}
  />

  <section class="desktop-content" aria-busy={state.loading || state.refreshing}>
    <div class="section-label">
      <span>OVERLAY INSTANCES</span>
      <small>{state.monitors.length} {state.monitors.length === 1 ? "display" : "displays"} connected</small>
    </div>
    {#if state.loading}
      <div class="overlay-list skeleton-list" aria-label="Loading overlays">
        {#each Array(3) as _}
          <div class="skeleton-row"><span></span><div><i></i><i></i><b></b></div></div>
        {/each}
      </div>
    {:else if state.loadError !== null}
      <div class="state-panel">
        <span class="state-icon error"><RefreshCw size={24} /></span>
        <div><h2>Desktop data could not be loaded</h2><p>{state.loadError}</p></div>
        <button class="secondary-button" type="button" onclick={onreload}>Try again</button>
      </div>
    {:else if state.overlays.length === 0}
      <div class="state-panel empty">
        <span class="state-icon"><Layers3 size={28} /></span>
        <div>
          <h2>Your overlay list is empty</h2>
          <p>Select an animation in Collection, then choose <strong>Show on desktop</strong>.</p>
        </div>
        <button class="primary-button" type="button" onclick={oncollection}>Open collection</button>
      </div>
    {:else}
      <div class="overlay-list">
        {#each state.overlays as item (item.overlay.id)}
          <DesktopOverlayCard
            {item}
            monitors={state.monitors}
            src={state.assetUrls[item.media.id]}
            recipe={state.editRecipes[item.media.id]}
            busy={state.busyIds.has(item.overlay.id)}
            {onupdate}
            {onmovetomonitor}
            {onduplicate}
            {onclose}
          />
        {/each}
      </div>
    {/if}
  </section>
</main>

<style>
.desktop-workspace { container: desktop-workspace / inline-size; min-width: 0; min-height: 0; overflow: auto; background: var(--canvas-950); }
.workspace-header { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; padding: 24px 24px 17px; background: radial-gradient(circle at 72% 0%, color-mix(in srgb, var(--brand-400) 12%, transparent), transparent 36%); }
.eyebrow, .bar-label, .section-label > span { color: var(--brand-300); font-size: 10px; font-weight: 750; letter-spacing: 0.11em; }
h1 { margin: 4px 0; font-family: "Space Grotesk Variable", sans-serif; font-size: 27px; line-height: 1.2; }
.workspace-header p { margin: 0; color: var(--ink-300); font-size: 12px; }
.desktop-summary { display: flex; align-items: center; gap: 8px; color: var(--ink-500); font-size: 10px; white-space: nowrap; }
.desktop-summary > span { display: grid; justify-items: center; gap: 1px; }
.desktop-summary strong { color: var(--ink-100); font-size: 17px; font-variant-numeric: tabular-nums; }
.desktop-summary i { width: 1px; height: 25px; background: var(--line-700); }
.desktop-summary button { display: grid; width: 34px; height: 34px; margin-left: 3px; place-items: center; color: var(--ink-300); background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 5px; cursor: pointer; }
.desktop-summary button:hover { color: var(--ink-100); background: var(--surface-800); }
.desktop-summary button:disabled { cursor: wait; opacity: 0.55; }
.browser-note { display: flex; align-items: center; gap: 10px; padding: 9px 24px; color: var(--ink-300); font-size: 11px; background: color-mix(in srgb, var(--warning-400) 8%, var(--surface-850)); border-top: 1px solid color-mix(in srgb, var(--warning-400) 28%, var(--line-700)); }
.browser-note :global(svg) { flex: 0 0 auto; color: var(--warning-400); }
.browser-note strong { color: var(--ink-100); }
.action-bar { display: flex; align-items: center; gap: 8px; margin: 0 24px 10px; padding: 8px; overflow-x: auto; background: var(--canvas-900); border: 1px solid var(--line-700); border-radius: 8px; }
.bar-label { padding-inline: 6px 9px; }
.action-group { display: flex; gap: 4px; padding-left: 8px; border-left: 1px solid var(--line-700); }
.action-bar button { display: inline-flex; min-height: 34px; align-items: center; justify-content: center; gap: 6px; padding-inline: 10px; color: var(--ink-300); font-size: 11px; font-weight: 650; white-space: nowrap; background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 5px; cursor: pointer; }
.action-bar button:hover { color: var(--ink-100); background: var(--surface-750); }
.action-bar button:disabled { cursor: default; opacity: 0.4; }
.danger-group { margin-left: auto; padding-left: 8px; border-left: 1px solid var(--line-700); }
.action-bar button.danger-action:hover { color: var(--danger-400); }
.close-confirm { display: flex; align-items: center; gap: 6px; }
.close-confirm small { color: var(--ink-300); font-size: 10px; white-space: nowrap; }
.action-bar button.danger-confirm { color: var(--canvas-950); background: var(--danger-400); border-color: var(--danger-400); }
.desktop-content { min-height: 0; padding: 17px 24px 28px; }
.section-label { display: flex; max-width: 1120px; align-items: center; justify-content: space-between; margin: 0 auto 8px; }
.section-label small { color: var(--ink-500); font-size: 10px; }
.overlay-list { display: grid; max-width: 1120px; gap: 9px; margin-inline: auto; }
.state-panel { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; max-width: 760px; align-items: center; gap: 20px; margin: 42px auto; padding: 24px; background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 10px; }
.state-panel.empty { background: linear-gradient(135deg, color-mix(in srgb, var(--brand-400) 9%, var(--surface-850)), var(--surface-850)); }
.state-icon { display: grid; width: 52px; height: 52px; place-items: center; color: var(--brand-300); background: var(--surface-750); border-radius: 10px; }
.state-icon.error { color: var(--danger-400); }
.state-panel h2 { margin: 0 0 4px; font-size: 18px; }
.state-panel p { margin: 0; color: var(--ink-300); font-size: 12px; }
.state-panel p strong { color: var(--ink-100); }
.primary-button, .secondary-button { display: inline-flex; min-height: 38px; align-items: center; justify-content: center; padding-inline: 14px; font-weight: 650; border-radius: 5px; cursor: pointer; }
.primary-button { color: var(--canvas-950); background: var(--brand-400); border: 0; }
.primary-button:hover { background: var(--brand-300); }
.secondary-button { color: var(--ink-300); background: var(--surface-800); border: 1px solid var(--line-700); }
.skeleton-row { display: grid; grid-template-columns: 160px 1fr; gap: 16px; padding: 10px; background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 7px; }
.skeleton-row span, .skeleton-row i, .skeleton-row b { display: block; background: linear-gradient(90deg, var(--surface-800), var(--surface-750), var(--surface-800)); background-size: 240% 100%; animation: shimmer 1.4s ease infinite; }
.skeleton-row span { aspect-ratio: 4 / 3; border-radius: 6px; }
.skeleton-row div { display: grid; align-content: center; gap: 14px; }
.skeleton-row i { width: 42%; height: 12px; border-radius: 3px; }
.skeleton-row i + i { width: 68%; height: 8px; }
.skeleton-row b { width: 100%; height: 5px; margin-top: 12px; border-radius: 3px; }
@keyframes shimmer { from { background-position: 100% 0; } to { background-position: -140% 0; } }

@container desktop-workspace (max-width: 880px) {
  .workspace-header { align-items: flex-start; }
  .desktop-summary i, .desktop-summary span:nth-of-type(3), .desktop-summary span:nth-of-type(4) { display: none; }
}

@media (max-width: 620px) {
  .workspace-header { gap: 12px; padding: 18px 16px 14px; }
  .workspace-header p { display: none; }
  .desktop-summary span:nth-of-type(2), .desktop-summary i { display: none; }
  .browser-note { padding-inline: 16px; }
  .action-bar { margin-inline: 16px; }
  .desktop-content { padding-inline: 16px; }
  .state-panel { grid-template-columns: 1fr; margin-block: 24px; text-align: center; }
  .state-icon { margin-inline: auto; }
}

@media (prefers-reduced-motion: reduce) {
  .skeleton-row span, .skeleton-row i, .skeleton-row b { animation: none; }
}
</style>
