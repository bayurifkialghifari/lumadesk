<script lang="ts">
import { Check, Copy, Layers3, Pencil, Play, RefreshCw, Save, Trash2, X } from "@lucide/svelte";
import type { DesktopSceneId, DesktopSceneSummary } from "../contracts";

export let scenes: readonly DesktopSceneSummary[];
export let overlayCount: number;
export let busy = false;
export let oncreate: (name: string) => void;
export let onapply: (id: DesktopSceneId) => void;
export let onupdate: (id: DesktopSceneId) => void;
export let onrename: (id: DesktopSceneId, name: string) => void;
export let onduplicate: (id: DesktopSceneId) => void;
export let ondelete: (id: DesktopSceneId) => void;

let sceneName = "";
let selectedId: DesktopSceneId | null = null;
let renameDraft = "";
let renaming = false;
let confirmingDelete = false;

$: if (selectedId === null || !scenes.some((scene) => scene.id === selectedId)) {
  selectedId = scenes[0]?.id ?? null;
}
$: selectedScene = scenes.find((scene) => scene.id === selectedId);

function saveCurrent(): void {
  const name = sceneName.trim();
  if (name.length === 0 || busy) return;
  oncreate(name);
  sceneName = "";
}

function selectScene(scene: DesktopSceneSummary): void {
  selectedId = scene.id;
  renaming = false;
  confirmingDelete = false;
}

function beginRename(): void {
  if (selectedScene === undefined) return;
  renameDraft = selectedScene.name;
  renaming = true;
  confirmingDelete = false;
}

function commitRename(): void {
  const name = renameDraft.trim();
  if (selectedId === null || name.length === 0 || busy) return;
  onrename(selectedId, name);
  renaming = false;
}
</script>

<section class="scene-manager" aria-labelledby="scene-manager-title">
  <div class="scene-heading">
    <div class="scene-icon"><Layers3 size={17} /></div>
    <div>
      <span>DESKTOP SCENES</span>
      <h2 id="scene-manager-title">Save and restore a layout</h2>
    </div>
  </div>

  <form class="save-scene" onsubmit={(event) => { event.preventDefault(); saveCurrent(); }}>
    <label for="scene-name">Scene name</label>
    <div>
      <input
        id="scene-name"
        maxlength="80"
        placeholder="Gaming, focus, streaming…"
        bind:value={sceneName}
        disabled={busy}
      />
      <button type="submit" disabled={busy || sceneName.trim().length === 0}>
        <Save size={15} /> Save current
      </button>
    </div>
    <small>Captures all {overlayCount} persisted overlays, including hidden and locked states.</small>
  </form>

  <div class="scene-browser">
    <div class="scene-list" aria-label="Saved desktop scenes">
      {#if scenes.length === 0}
        <div class="scene-empty">No saved scenes yet.</div>
      {:else}
        {#each scenes as scene (scene.id)}
          <button
            class:selected={scene.id === selectedId}
            type="button"
            onclick={() => selectScene(scene)}
          >
            <span>{scene.name}</span>
            <small>{scene.itemCount} {scene.itemCount === 1 ? "overlay" : "overlays"}</small>
          </button>
        {/each}
      {/if}
    </div>

    <div class="scene-actions">
      {#if selectedScene === undefined}
        <p>Select a saved scene to manage it.</p>
      {:else}
        <div class="selected-scene">
          <small>SELECTED SCENE</small>
          {#if renaming}
            <div class="rename-row">
              <input maxlength="80" aria-label="New scene name" bind:value={renameDraft} disabled={busy} />
              <button class="icon-button primary" type="button" title="Save name" aria-label="Save scene name" disabled={busy || renameDraft.trim().length === 0} onclick={commitRename}><Check size={15} /></button>
              <button class="icon-button" type="button" title="Cancel rename" aria-label="Cancel scene rename" disabled={busy} onclick={() => (renaming = false)}><X size={15} /></button>
            </div>
          {:else}
            <strong>{selectedScene.name}</strong>
          {/if}
        </div>

        <div class="primary-actions">
          <button class="apply" type="button" disabled={busy} onclick={() => selectedId !== null && onapply(selectedId)}>
            <Play size={15} fill="currentColor" /> Apply scene
          </button>
          <button type="button" disabled={busy} onclick={() => selectedId !== null && onupdate(selectedId)}>
            <RefreshCw size={15} /> Update snapshot
          </button>
        </div>

        <div class="secondary-actions">
          <button type="button" disabled={busy || renaming} onclick={beginRename}><Pencil size={14} /> Rename</button>
          <button type="button" disabled={busy} onclick={() => selectedId !== null && onduplicate(selectedId)}><Copy size={14} /> Duplicate</button>
          {#if confirmingDelete}
            <span class="delete-confirm">
              <small>Delete this scene?</small>
              <button class="danger" type="button" disabled={busy} onclick={() => { confirmingDelete = false; if (selectedId !== null) ondelete(selectedId); }}>Delete</button>
              <button type="button" disabled={busy} onclick={() => (confirmingDelete = false)}>Cancel</button>
            </span>
          {:else}
            <button class="delete" type="button" disabled={busy} onclick={() => { confirmingDelete = true; renaming = false; }}><Trash2 size={14} /> Delete</button>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</section>

<style>
.scene-manager { margin: 0 24px; overflow: hidden; background: var(--canvas-900); border: 1px solid var(--line-700); border-radius: 8px; }
.scene-heading { display: flex; align-items: center; gap: 10px; padding: 13px 14px; border-bottom: 1px solid var(--line-700); }
.scene-icon { display: grid; width: 32px; height: 32px; place-items: center; color: var(--brand-300); background: color-mix(in srgb, var(--brand-400) 12%, var(--surface-800)); border: 1px solid color-mix(in srgb, var(--brand-400) 36%, var(--line-700)); border-radius: 5px; }
.scene-heading span, .selected-scene > small { color: var(--ink-500); font-size: 9px; font-weight: 750; letter-spacing: 0.11em; }
h2 { margin: 2px 0 0; font-size: 14px; }
.save-scene { display: grid; grid-template-columns: 100px minmax(280px, 520px) minmax(180px, 1fr); align-items: center; gap: 12px; padding: 12px 14px; border-bottom: 1px solid var(--line-700); }
.save-scene > label { color: var(--ink-300); font-size: 11px; font-weight: 650; }
.save-scene > div { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 6px; }
input { min-width: 0; height: 36px; padding: 0 10px; color: var(--ink-100); background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 5px; }
input::placeholder { color: var(--ink-500); }
.save-scene button, .scene-actions button { display: inline-flex; min-height: 36px; align-items: center; justify-content: center; gap: 6px; padding: 0 11px; color: var(--ink-300); font-size: 11px; font-weight: 700; background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 5px; cursor: pointer; }
.save-scene button { color: var(--canvas-950); background: var(--brand-400); border-color: var(--brand-400); }
.save-scene small { color: var(--ink-500); font-size: 10px; line-height: 1.4; }
button:hover { color: var(--ink-100); background: var(--surface-750); }
button:disabled, input:disabled { cursor: default; opacity: 0.5; }
.scene-browser { display: grid; grid-template-columns: minmax(220px, 0.8fr) minmax(360px, 1.2fr); min-height: 164px; }
.scene-list { display: grid; max-height: 220px; align-content: start; gap: 2px; padding: 8px; overflow: auto; border-right: 1px solid var(--line-700); }
.scene-list > button { display: grid; grid-template-columns: minmax(0, 1fr) auto; align-items: center; gap: 12px; width: 100%; min-height: 38px; padding: 0 10px; color: var(--ink-300); text-align: left; background: transparent; border: 1px solid transparent; border-radius: 4px; cursor: pointer; }
.scene-list > button:hover { background: var(--surface-850); }
.scene-list > button.selected { color: var(--ink-100); background: var(--surface-800); border-color: color-mix(in srgb, var(--brand-400) 48%, var(--line-700)); }
.scene-list span { overflow: hidden; font-size: 12px; font-weight: 650; text-overflow: ellipsis; white-space: nowrap; }
.scene-list small { color: var(--ink-500); font-size: 10px; }
.scene-empty { padding: 22px 12px; color: var(--ink-500); font-size: 11px; text-align: center; }
.scene-actions { display: grid; align-content: center; gap: 11px; padding: 14px; }
.scene-actions > p { margin: 0; color: var(--ink-500); font-size: 12px; text-align: center; }
.selected-scene { display: grid; gap: 3px; }
.selected-scene > strong { overflow: hidden; font-size: 15px; text-overflow: ellipsis; white-space: nowrap; }
.rename-row { display: grid; grid-template-columns: minmax(0, 1fr) 36px 36px; gap: 5px; }
.scene-actions .icon-button { width: 36px; padding: 0; }
.scene-actions .icon-button.primary { color: var(--canvas-950); background: var(--brand-400); border-color: var(--brand-400); }
.primary-actions, .secondary-actions, .delete-confirm { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
.scene-actions button.apply { color: var(--canvas-950); background: var(--brand-400); border-color: var(--brand-400); }
.scene-actions button.apply:hover, .save-scene button:hover { background: var(--brand-300); }
.scene-actions button.delete:hover, .scene-actions button.danger { color: var(--danger-400); }
.delete-confirm { padding-left: 8px; border-left: 1px solid var(--line-700); }
.delete-confirm small { color: var(--ink-500); font-size: 10px; }

@container desktop-workspace (max-width: 820px) {
  .save-scene { grid-template-columns: 1fr; }
  .save-scene > label { display: none; }
  .scene-browser { grid-template-columns: 1fr; }
  .scene-list { border-right: 0; border-bottom: 1px solid var(--line-700); }
}

@media (max-width: 620px) {
  .scene-manager { margin-inline: 16px; }
  .save-scene > div { grid-template-columns: 1fr; }
}
</style>
