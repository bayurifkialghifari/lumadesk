<script lang="ts">
import { CloudOff, Layers3, MousePointer2, Pause, RefreshCw, Trash2 } from "@lucide/svelte";
import type { OverlayId, OverlayPatch } from "../contracts";
import type { DesktopState } from "../desktop-model";
import DesktopOverlayCard from "./DesktopOverlayCard.svelte";

export let state: DesktopState;
export let oneditmode: (enabled: boolean) => void;
export let onupdate: (id: OverlayId, patch: OverlayPatch) => void;
export let onduplicate: (id: OverlayId) => void;
export let onclose: (id: OverlayId) => void;
export let onpauseall: () => void;
export let oncloseall: () => void;
export let onreload: () => void;
export let oncollection: () => void;

let confirmingCloseAll = false;
$: playingCount = state.overlays.filter((item) => item.overlay.isPlaying).length;
</script>

<main class="desktop-workspace">
  <header class="workspace-header">
    <div>
      <span class="eyebrow">OVERLAY BERJALAN</span>
      <h1>Aktif di Desktop</h1>
      <p>Atur setiap instance tanpa mengubah file asli di koleksi.</p>
    </div>
    <div class="desktop-summary">
      <strong>{state.overlays.length}</strong>
      <span>aktif</span>
      <i></i>
      <strong>{playingCount}</strong>
      <span>diputar</span>
      <button type="button" aria-label="Muat ulang overlay" title="Muat ulang" disabled={state.refreshing} onclick={onreload}>
        <RefreshCw size={16} />
      </button>
    </div>
  </header>

  {#if !state.nativeAvailable}
    <div class="browser-note" role="status">
      <CloudOff size={17} />
      <span><strong>Mode pratinjau browser.</strong> Window overlay dan kontrol native aktif di aplikasi desktop.</span>
    </div>
  {/if}

  <section class="action-bar" aria-label="Aksi overlay desktop">
    <button
      class:active={state.editMode}
      class="edit-button"
      type="button"
      aria-pressed={state.editMode}
      disabled={state.editModeBusy || !state.nativeAvailable}
      onclick={() => oneditmode(!state.editMode)}
    >
      <MousePointer2 size={16} />
      <span><strong>Mode edit</strong><small>{state.editMode ? "Overlay dapat digeser" : "Klik menembus overlay"}</small></span>
      <i aria-hidden="true"><b></b></i>
    </button>

    <div class="batch-actions">
      <button type="button" disabled={state.batchBusy || playingCount === 0} onclick={onpauseall}>
        <Pause size={16} /> Jeda semua
      </button>
      {#if confirmingCloseAll}
        <span class="close-confirm">
          <small>Tutup {state.overlays.length} overlay?</small>
          <button class="danger-confirm" type="button" disabled={state.batchBusy} onclick={() => { confirmingCloseAll = false; oncloseall(); }}>Ya, tutup</button>
          <button type="button" onclick={() => (confirmingCloseAll = false)}>Batal</button>
        </span>
      {:else}
        <button class="danger-action" type="button" disabled={state.batchBusy || state.overlays.length === 0} onclick={() => (confirmingCloseAll = true)}>
          <Trash2 size={16} /> Tutup semua
        </button>
      {/if}
    </div>
  </section>

  <section class="desktop-content" aria-busy={state.loading || state.refreshing}>
    {#if state.loading}
      <div class="overlay-list skeleton-list" aria-label="Memuat overlay aktif">
        {#each Array(3) as _}
          <div class="skeleton-row"><span></span><div><i></i><i></i><b></b></div></div>
        {/each}
      </div>
    {:else if state.loadError !== null}
      <div class="state-panel">
        <span class="state-icon error"><RefreshCw size={24} /></span>
        <div><h2>Overlay belum terbaca</h2><p>{state.loadError}</p></div>
        <button class="secondary-button" type="button" onclick={onreload}>Coba lagi</button>
      </div>
    {:else if state.overlays.length === 0}
      <div class="state-panel empty">
        <span class="state-icon"><Layers3 size={28} /></span>
        <div>
          <h2>Desktop masih kosong</h2>
          <p>Pilih animasi di Koleksi, lalu tekan <strong>Tampilkan di desktop</strong>.</p>
        </div>
        <button class="primary-button" type="button" onclick={oncollection}>Buka koleksi</button>
      </div>
    {:else}
      <div class="overlay-list">
        {#each state.overlays as item (item.overlay.id)}
          <DesktopOverlayCard
            {item}
            src={state.assetUrls[item.media.id]}
            busy={state.busyIds.has(item.overlay.id)}
            {onupdate}
            {onduplicate}
            {onclose}
          />
        {/each}
      </div>
    {/if}
  </section>
</main>

<style>
.desktop-workspace {
  container: desktop-workspace / inline-size;
  min-width: 0;
  min-height: 0;
  overflow: auto;
  background: var(--canvas-950);
}

.workspace-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 24px;
  padding: 28px 24px 20px;
  background: radial-gradient(circle at 72% 0%, color-mix(in srgb, var(--brand-400) 12%, transparent), transparent 36%);
}
.eyebrow { color: var(--brand-300); font-size: 12px; font-weight: 700; letter-spacing: 0.08em; }
h1 { margin: 4px 0; font-family: "Space Grotesk Variable", sans-serif; font-size: 28px; line-height: 1.2; }
.workspace-header p { margin: 0; color: var(--ink-300); font-size: 13px; }

.desktop-summary { display: flex; align-items: baseline; gap: 7px; color: var(--ink-500); font-size: 12px; white-space: nowrap; }
.desktop-summary strong { color: var(--ink-100); font-size: 20px; }
.desktop-summary i { width: 1px; height: 20px; margin-inline: 5px; background: var(--line-700); }
.desktop-summary button { display: grid; width: 40px; height: 40px; margin-left: 10px; place-items: center; color: var(--ink-300); background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 6px; cursor: pointer; }
.desktop-summary button:hover { color: var(--ink-100); background: var(--surface-800); }
.desktop-summary button:disabled { cursor: wait; opacity: 0.55; }

.browser-note { display: flex; align-items: center; gap: 10px; padding: 10px 24px; color: var(--ink-300); font-size: 12px; background: color-mix(in srgb, var(--warning-400) 8%, var(--surface-850)); border-top: 1px solid color-mix(in srgb, var(--warning-400) 28%, var(--line-700)); }
.browser-note :global(svg) { flex: 0 0 auto; color: var(--warning-400); }
.browser-note strong { color: var(--ink-100); }

.action-bar { display: flex; align-items: center; justify-content: space-between; gap: 16px; margin: 0 24px; padding: 12px; background: var(--canvas-900); border: 1px solid var(--line-700); border-radius: 8px; }
.edit-button { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; min-width: 250px; align-items: center; gap: 10px; padding: 7px 10px; color: var(--ink-300); text-align: left; background: transparent; border: 0; border-radius: 6px; cursor: pointer; }
.edit-button:hover { background: var(--surface-850); }
.edit-button.active { color: var(--brand-300); background: color-mix(in srgb, var(--brand-400) 9%, var(--surface-850)); }
.edit-button > span { display: grid; gap: 2px; }
.edit-button strong { color: var(--ink-100); font-size: 13px; }
.edit-button small { color: var(--ink-500); font-size: 11px; }
.edit-button > i { display: flex; width: 34px; height: 18px; align-items: center; padding: 2px; background: var(--surface-750); border-radius: 999px; transition: background 140ms ease; }
.edit-button > i b { width: 14px; height: 14px; background: var(--ink-300); border-radius: 50%; transition: transform 140ms ease; }
.edit-button.active > i { background: var(--brand-400); }
.edit-button.active > i b { background: var(--canvas-950); transform: translateX(16px); }
.edit-button:disabled { cursor: not-allowed; opacity: 0.55; }

.batch-actions, .close-confirm { display: flex; align-items: center; gap: 8px; }
.batch-actions > button, .close-confirm button { display: inline-flex; min-height: 40px; align-items: center; justify-content: center; gap: 8px; padding-inline: 14px; color: var(--ink-300); font-weight: 650; background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 6px; cursor: pointer; }
.batch-actions > button:hover, .close-confirm button:hover { color: var(--ink-100); background: var(--surface-750); }
.batch-actions > button.danger-action:hover { color: var(--danger-400); }
.batch-actions button:disabled { cursor: default; opacity: 0.45; }
.close-confirm { padding-left: 12px; border-left: 1px solid var(--line-700); }
.close-confirm small { color: var(--ink-300); }
.close-confirm button.danger-confirm { color: var(--canvas-950); background: var(--danger-400); border-color: var(--danger-400); }

.desktop-content { min-height: 0; padding: 20px 24px 28px; }
.overlay-list { display: grid; max-width: 1120px; gap: 12px; margin-inline: auto; }
.state-panel { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; max-width: 760px; align-items: center; gap: 20px; margin: 56px auto; padding: 24px; background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 12px; }
.state-panel.empty { background: linear-gradient(135deg, color-mix(in srgb, var(--brand-400) 9%, var(--surface-850)), var(--surface-850)); }
.state-icon { display: grid; width: 56px; height: 56px; place-items: center; color: var(--brand-300); background: var(--surface-750); border-radius: 12px; }
.state-icon.error { color: var(--danger-400); }
.state-panel h2 { margin: 0 0 4px; font-size: 20px; }
.state-panel p { margin: 0; color: var(--ink-300); font-size: 13px; }
.state-panel p strong { color: var(--ink-100); }
.primary-button, .secondary-button { display: inline-flex; min-height: 40px; align-items: center; justify-content: center; padding-inline: 16px; font-weight: 650; border-radius: 6px; cursor: pointer; }
.primary-button { color: var(--canvas-950); background: var(--brand-400); border: 0; }
.primary-button:hover { background: var(--brand-300); }
.secondary-button { color: var(--ink-300); background: var(--surface-800); border: 1px solid var(--line-700); }

.skeleton-row { display: grid; grid-template-columns: 184px 1fr; gap: 16px; padding: 12px; background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 8px; }
.skeleton-row span, .skeleton-row i, .skeleton-row b { display: block; background: linear-gradient(90deg, var(--surface-800), var(--surface-750), var(--surface-800)); background-size: 240% 100%; animation: shimmer 1.4s ease infinite; }
.skeleton-row span { aspect-ratio: 4 / 3; border-radius: 8px; }
.skeleton-row div { display: grid; align-content: center; gap: 16px; }
.skeleton-row i { width: 42%; height: 14px; border-radius: 4px; }
.skeleton-row i + i { width: 68%; height: 9px; }
.skeleton-row b { width: 100%; height: 5px; margin-top: 16px; border-radius: 4px; }
@keyframes shimmer { from { background-position: 100% 0; } to { background-position: -140% 0; } }

@container desktop-workspace (max-width: 780px) {
  .action-bar { align-items: stretch; flex-direction: column; }
  .batch-actions { justify-content: flex-end; }
}

@media (max-width: 620px) {
  .workspace-header { align-items: flex-start; gap: 12px; padding: 20px 16px 16px; }
  .workspace-header p, .desktop-summary strong:nth-of-type(2), .desktop-summary span:nth-of-type(2), .desktop-summary i { display: none; }
  .desktop-summary button { margin-left: 4px; }
  .browser-note { padding-inline: 16px; }
  .action-bar { margin-inline: 16px; }
  .edit-button { min-width: 0; }
  .batch-actions, .close-confirm { align-items: stretch; flex-direction: column; }
  .close-confirm { padding-top: 8px; padding-left: 0; border-top: 1px solid var(--line-700); border-left: 0; }
  .desktop-content { padding-inline: 16px; }
  .state-panel { grid-template-columns: 1fr; margin-block: 24px; text-align: center; }
  .state-icon { margin-inline: auto; }
}

@media (prefers-reduced-motion: reduce) {
  .edit-button > i, .edit-button > i b { transition-duration: 0.01ms; }
  .skeleton-row span, .skeleton-row i, .skeleton-row b { animation: none; }
}
</style>
