<script lang="ts">
import {
  Bot,
  Database,
  Download,
  FolderOpen,
  LoaderCircle,
  Monitor,
  Palette,
  Save,
  Settings2,
  Trash2,
} from "@lucide/svelte";
import { onMount } from "svelte";
import { downloadAiModel, listAiModels, removeAiModel } from "../api";
import type { AiModel, AiModelStatus, AppSettings } from "../contracts";
import { userErrorMessage } from "../presentation";

export let settings: AppSettings;
export let nativeAvailable: boolean;
export let onsave: (settings: AppSettings) => Promise<boolean>;
export let onopenfolder: () => Promise<void>;
export let onclearcache: () => Promise<void>;

let draft = { ...settings };
let previousSettings = settings;
let busy = false;
let storageAction: "folder" | "cache" | null = null;
let models: readonly AiModelStatus[] = [];
let modelAction: AiModel | null = null;
let modelError: string | null = null;

onMount(() => {
  if (nativeAvailable) void refreshModels();
});

$: if (settings !== previousSettings) {
  previousSettings = settings;
  draft = { ...settings };
}

function numberValue(event: Event): number {
  return Number((event.currentTarget as HTMLInputElement).value);
}

async function save(): Promise<void> {
  if (busy) return;
  busy = true;
  await onsave(draft);
  busy = false;
}

async function runStorageAction(
  action: "folder" | "cache",
  operation: () => Promise<void>,
): Promise<void> {
  if (!nativeAvailable || storageAction !== null) return;
  storageAction = action;
  try {
    await operation();
  } finally {
    storageAction = null;
  }
}

async function refreshModels(): Promise<void> {
  try {
    models = await listAiModels();
    modelError = null;
  } catch (error) {
    modelError = userErrorMessage(error, "Local AI model status could not be loaded.");
  }
}

function modelStatus(model: AiModel): AiModelStatus | undefined {
  return models.find((candidate) => candidate.model === model);
}

async function runModelAction(model: AiModel, action: "download" | "remove"): Promise<void> {
  if (!nativeAvailable || modelAction !== null) return;
  modelAction = model;
  modelError = null;
  try {
    const status =
      action === "download" ? await downloadAiModel(model) : await removeAiModel(model);
    models = [...models.filter((candidate) => candidate.model !== model), status];
  } catch (error) {
    const message = userErrorMessage(
      error,
      action === "download"
        ? "The local AI model could not be downloaded."
        : "The local AI model could not be removed.",
    );
    await refreshModels();
    modelError = message;
  } finally {
    modelAction = null;
  }
}

function formatBytes(bytes: number): string {
  return `${(bytes / 1_000_000).toFixed(bytes < 10_000_000 ? 1 : 0)} MB`;
}
</script>

<main class="settings-workspace">
  <header class="workspace-header">
    <div>
      <span class="eyebrow">PREFERENCES</span>
      <h1>Settings</h1>
      <p>Control startup, overlays, storage, and local processing.</p>
    </div>
    <button class="save-button" type="button" disabled={busy} onclick={() => void save()}><Save size={17} /> {busy ? "Saving…" : "Save settings"}</button>
  </header>

  <div class="settings-grid">
    <section>
      <div class="section-heading"><Palette size={18} /><div><h2>Appearance</h2><p>Application theme and collection layout.</p></div></div>
      <label class="field"><span>Theme</span><select bind:value={draft.theme}><option value="dark">Dark</option><option value="system">Follow system</option></select></label>
      <label class="field"><span>Collection view</span><select bind:value={draft.collectionView}><option value="grid">Grid</option><option value="list">Compact list</option></select></label>
    </section>

    <section>
      <div class="section-heading"><Monitor size={18} /><div><h2>Desktop behavior</h2><p>Startup and window lifecycle.</p></div></div>
      <label class="toggle-row"><input type="checkbox" bind:checked={draft.launchAtLogin} /><span><strong>Launch at login</strong><small>Start LumaDesk after you sign in.</small></span></label>
      <label class="toggle-row"><input type="checkbox" bind:checked={draft.closeToTray} /><span><strong>Close to tray</strong><small>Keep overlays running when the main window closes.</small></span></label>
      <label class="toggle-row"><input type="checkbox" bind:checked={draft.restoreOverlays} /><span><strong>Restore overlays</strong><small>Open the last active desktop layout at launch.</small></span></label>
    </section>

    <section>
      <div class="section-heading"><Settings2 size={18} /><div><h2>Overlay defaults</h2><p>Starting values for new desktop items.</p></div></div>
      <label class="range-row"><span>Width <output>{draft.defaultOverlayWidth} px</output></span><input type="range" min="64" max="1024" step="16" value={draft.defaultOverlayWidth} oninput={(event) => (draft.defaultOverlayWidth = numberValue(event))} /></label>
      <label class="range-row"><span>Opacity <output>{Math.round(draft.defaultOpacity * 100)}%</output></span><input type="range" min="0.1" max="1" step="0.05" value={draft.defaultOpacity} oninput={(event) => (draft.defaultOpacity = numberValue(event))} /></label>
      <label class="range-row"><span>Speed <output>{draft.defaultSpeed.toLocaleString("en-US")}×</output></span><input type="range" min="0.25" max="4" step="0.25" value={draft.defaultSpeed} oninput={(event) => (draft.defaultSpeed = numberValue(event))} /></label>
    </section>

    <section>
      <div class="section-heading"><Database size={18} /><div><h2>Storage</h2><p>Open local files or remove regenerable data.</p></div></div>
      <div class="button-row">
        <button type="button" disabled={!nativeAvailable || storageAction !== null} onclick={() => void runStorageAction("folder", onopenfolder)}><FolderOpen size={16} /> {storageAction === "folder" ? "Opening…" : "Open data folder"}</button>
        <button type="button" disabled={!nativeAvailable || storageAction !== null} onclick={() => void runStorageAction("cache", onclearcache)}><Database size={16} /> {storageAction === "cache" ? "Clearing…" : "Clear cache"}</button>
      </div>
    </section>

    <section class="wide">
      <div class="section-heading"><Bot size={18} /><div><h2>Local AI models</h2><p>Install only the background models you want to use in Creator Studio.</p></div></div>
      <div class="model-grid">
        <article class="model-card built-in">
          <div class="model-select">
            <span class="model-title"><strong>Edge removal</strong></span>
            <span>Available immediately for simple, solid backgrounds.</span>
            <small>Built in · no download</small>
          </div>
        </article>
        {#each [["fast", "Fast model", "Small model for responsive previews and overlays."], ["quality", "Quality model", "Larger model for more detailed subject masks."]] as entry}
          {@const model = entry[0] as AiModel}
          {@const status = modelStatus(model)}
          <article class="model-card" class:installed={status?.status === "installed"}>
            <div class="model-select">
              <span class="model-title"><strong>{entry[1]}</strong></span>
              <span>{entry[2]}</span>
              <small>{status === undefined ? "Checking…" : `${formatBytes(status.sizeBytes)} · ${status.status}`}</small>
            </div>
            <div class="model-action">
              {#if status?.status === "installed"}
                <button type="button" disabled={modelAction !== null} onclick={() => void runModelAction(model, "remove")}><Trash2 size={14} /> Remove</button>
              {:else}
                <button type="button" disabled={!nativeAvailable || modelAction !== null || status?.status === "downloading" || status?.status === "removing"} onclick={() => void runModelAction(model, "download")}>
                  {#if status?.status === "removing"}<LoaderCircle class="spin" size={14} /> Removing…{:else if modelAction === model || status?.status === "downloading"}<LoaderCircle class="spin" size={14} /> Downloading…{:else}<Download size={14} /> Download{/if}
                </button>
              {/if}
            </div>
          </article>
        {/each}
      </div>
      {#if modelError !== null}<p class="model-error" role="alert">{modelError}</p>{/if}
      <p class="local-note">Models are optional and removable. Every download is verified before local inference starts.</p>
    </section>
  </div>
</main>

<style>
.settings-workspace { min-width: 0; min-height: 0; overflow: auto; background: var(--canvas-950); }
.workspace-header { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; padding: 28px 24px 20px; border-bottom: 1px solid var(--line-700); }
.eyebrow { color: var(--brand-300); font-size: 11px; font-weight: 750; letter-spacing: .09em; }
h1 { margin: 4px 0; font-family: "Space Grotesk Variable", sans-serif; font-size: 28px; }
.workspace-header p { margin: 0; color: var(--ink-500); font-size: 13px; }
.save-button { display: inline-flex; min-height: 40px; align-items: center; gap: 8px; padding: 0 16px; color: var(--canvas-950); font-weight: 750; background: var(--brand-400); border: 1px solid var(--brand-400); border-radius: 6px; cursor: pointer; }
.save-button:hover { background: var(--brand-300); }
.save-button:disabled { cursor: wait; opacity: .55; }
.settings-grid { display: grid; grid-template-columns: repeat(2, minmax(300px, 1fr)); gap: 16px; max-width: 1120px; padding: 24px; }
section { padding: 20px; background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 8px; }
section.wide { grid-column: 1 / -1; }
.section-heading { display: flex; align-items: flex-start; gap: 12px; margin-bottom: 20px; color: var(--brand-300); }
.section-heading h2 { margin: 0; color: var(--ink-100); font-size: 15px; }
.section-heading p { margin: 3px 0 0; color: var(--ink-500); font-size: 12px; }
.field { display: grid; grid-template-columns: minmax(120px, 1fr) minmax(160px, 220px); align-items: center; gap: 16px; min-height: 48px; color: var(--ink-300); font-size: 12px; border-top: 1px solid var(--line-700); }
.field:first-of-type { border-top: 0; }
select { height: 36px; padding: 0 10px; color: var(--ink-100); background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 6px; }
.toggle-row { display: flex; align-items: center; gap: 12px; min-height: 54px; cursor: pointer; border-top: 1px solid var(--line-700); }
.toggle-row:first-of-type { border-top: 0; }
.toggle-row input { width: 18px; height: 18px; accent-color: var(--brand-400); }
.toggle-row span { display: grid; gap: 3px; }
.toggle-row strong { font-size: 12px; }
.toggle-row small { color: var(--ink-500); font-size: 11px; }
.range-row { display: grid; gap: 9px; margin-top: 17px; }
.range-row > span { display: flex; justify-content: space-between; color: var(--ink-300); font-size: 12px; }
.range-row input { width: 100%; accent-color: var(--brand-400); }
output { color: var(--ink-500); font-variant-numeric: tabular-nums; }
.button-row { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 20px; }
.button-row button { display: inline-flex; min-height: 38px; align-items: center; gap: 7px; padding: 0 12px; color: var(--ink-300); background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 6px; cursor: pointer; }
.button-row button:hover { color: var(--ink-100); background: var(--surface-750); }
.button-row button:disabled { cursor: not-allowed; opacity: .55; }
.model-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
.model-card { display: grid; overflow: hidden; color: var(--ink-300); background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 7px; }
.model-card:hover, .model-card.installed, .model-card.built-in { color: var(--ink-100); background: var(--surface-750); border-color: color-mix(in srgb, var(--brand-400) 50%, var(--line-700)); }
.model-select { display: grid; min-height: 118px; align-content: start; gap: 7px; padding: 16px; color: inherit; text-align: left; background: transparent; border: 0; cursor: pointer; }
.model-card .model-select { cursor: default; }
.model-title { display: flex; align-items: center; justify-content: space-between; color: var(--ink-100); }
.model-grid strong { font-size: 13px; }
.model-select > span:not(.model-title) { color: var(--ink-500); font-size: 11px; line-height: 1.45; }
.model-grid small { margin-top: auto; color: var(--brand-300); font-size: 11px; font-weight: 700; }
.model-action { display: flex; justify-content: flex-end; padding: 10px 12px; border-top: 1px solid var(--line-700); }
.model-action button { display: inline-flex; min-height: 30px; align-items: center; gap: 6px; padding: 0 9px; color: var(--ink-300); font-size: 11px; background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 5px; cursor: pointer; }
.model-action button:hover { color: var(--ink-100); }
.model-action button:disabled { cursor: wait; opacity: .55; }
.model-error { margin: 14px 0 0; color: var(--danger-300, #ff8c91); font-size: 11px; }
.local-note { margin: 14px 0 0; color: var(--ink-500); font-size: 11px; }
.model-action :global(.spin) { animation: spin 900ms linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 860px) { .settings-grid { grid-template-columns: 1fr; } section.wide { grid-column: auto; } .model-grid { grid-template-columns: 1fr; } }
@media (max-width: 560px) { .workspace-header { align-items: flex-start; flex-direction: column; } .settings-grid { padding: 14px; } .field { grid-template-columns: 1fr; gap: 7px; padding: 10px 0; } }
</style>
