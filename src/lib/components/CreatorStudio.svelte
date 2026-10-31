<script lang="ts">
import {
  Check,
  Crop,
  FlipHorizontal2,
  Grid3X3,
  LoaderCircle,
  RotateCcw,
  RotateCw,
  Save,
  Sparkles,
  Undo2,
} from "@lucide/svelte";
import { getEditRecipe, getSpriteSequence, saveCreatorChanges } from "../api";
import type {
  BackgroundRemovalMode,
  EditRecipe,
  MediaId,
  MediaItem,
  SpritePlaybackMode,
  SpriteSequence,
} from "../contracts";
import { defaultEditRecipe, recipeIsDefault } from "../edit-recipes";
import { mediaDimensions, mediaFormatLabel } from "../presentation";
import EditedMediaPreview from "./EditedMediaPreview.svelte";

export let items: readonly MediaItem[];
export let assetUrls: Readonly<Record<string, string>>;
export let selectedId: MediaId | null;
export let nativeAvailable: boolean;
export let onselect: (id: MediaId) => void;
export let onnotify: (tone: "success" | "error" | "info", message: string) => void;
export let onreload: () => void;

let recipe: EditRecipe | null = null;
let sequence: SpriteSequence | null = null;
let loadedId: MediaId | null = null;
let busy = false;
let loadError: string | null = null;
let spriteEnabled = false;
let columns = 4;
let rows = 4;
let frameCount = 16;
let frameDurationMs = 100;
let playbackMode: SpritePlaybackMode = "loop";

$: selected =
  selectedId === null ? items[0] : (items.find((item) => item.id === selectedId) ?? items[0]);
$: if (selected !== undefined && selected.id !== loadedId) void loadEditor(selected);
$: activeRecipe = recipe ?? (selected === undefined ? null : defaultEditRecipe(selected.id));
$: spriteCapacity = Math.max(1, columns * rows);
$: if (frameCount > spriteCapacity) frameCount = spriteCapacity;
$: spriteEligible =
  selected !== undefined &&
  (selected.assetKind === "still" || selected.assetKind === "sprite_sheet");
$: frameWidth =
  selected === undefined
    ? 1
    : spriteEnabled
      ? Math.max(1, Math.floor(selected.width / Math.max(1, columns)))
      : selected.width;
$: frameHeight =
  selected === undefined
    ? 1
    : spriteEnabled
      ? Math.max(1, Math.floor(selected.height / Math.max(1, rows)))
      : selected.height;
$: nativeFramePreview =
  nativeAvailable &&
  ((!spriteEnabled && sequence === null) ||
    (spriteEnabled &&
      sequence !== null &&
      sequence.columns === columns &&
      sequence.rows === rows &&
      sequence.frameCount === frameCount &&
      sequence.frameDurationMs === frameDurationMs &&
      sequence.playbackMode === playbackMode));

async function loadEditor(item: MediaItem): Promise<void> {
  loadedId = item.id;
  loadError = null;
  recipe = defaultEditRecipe(item.id);
  sequence = null;
  spriteEnabled = item.assetKind === "sprite_sheet";
  if (!nativeAvailable) return;
  try {
    const [nextRecipe, nextSequence] = await Promise.all([
      getEditRecipe(item.id),
      getSpriteSequence(item.id),
    ]);
    if (loadedId !== item.id) return;
    recipe = nextRecipe;
    sequence = nextSequence;
    spriteEnabled = nextSequence !== null;
    if (nextSequence !== null) {
      columns = nextSequence.columns;
      rows = nextSequence.rows;
      frameCount = nextSequence.frameCount;
      frameDurationMs = nextSequence.frameDurationMs;
      playbackMode = nextSequence.playbackMode;
    }
  } catch (error) {
    loadError = error instanceof Error ? error.message : "The editor data could not be loaded.";
  }
}

function patchRecipe(patch: Partial<EditRecipe>): void {
  if (activeRecipe === null) return;
  recipe = { ...activeRecipe, ...patch };
}

function rotate(direction: 1 | -1): void {
  if (activeRecipe === null) return;
  const steps = [0, 90, 180, 270] as const;
  const index = steps.indexOf(activeRecipe.rotationDegrees);
  const next = steps[(index + direction + steps.length) % steps.length] ?? 0;
  patchRecipe({ rotationDegrees: next });
}

function resetRecipe(): void {
  if (selected === undefined) return;
  recipe = defaultEditRecipe(selected.id);
}

function setFullCrop(enabled: boolean): void {
  if (selected === undefined) return;
  patchRecipe({
    crop: enabled
      ? {
          x: Math.round(frameWidth * 0.1),
          y: Math.round(frameHeight * 0.1),
          width: Math.max(1, Math.round(frameWidth * 0.8)),
          height: Math.max(1, Math.round(frameHeight * 0.8)),
        }
      : null,
  });
}

function setSpriteEnabled(enabled: boolean): void {
  if (!spriteEligible) return;
  spriteEnabled = enabled;
  clampCropToFrame();
}

function setSpriteGrid(axis: "columns" | "rows", value: number): void {
  if (selected === undefined) return;
  const sourceLimit = axis === "columns" ? selected.width : selected.height;
  const normalized = Math.max(1, Math.min(64, sourceLimit, Math.floor(value || 1)));
  if (axis === "columns") columns = normalized;
  else rows = normalized;
  frameCount = Math.min(frameCount, Math.max(1, columns * rows));
  clampCropToFrame();
}

function clampCropToFrame(): void {
  if (activeRecipe?.crop === null || activeRecipe === null) return;
  const bounds = currentFrameDimensions();
  const x = Math.min(Math.max(0, activeRecipe.crop.x), Math.max(0, bounds.width - 1));
  const y = Math.min(Math.max(0, activeRecipe.crop.y), Math.max(0, bounds.height - 1));
  patchRecipe({
    crop: {
      x,
      y,
      width: Math.max(1, Math.min(activeRecipe.crop.width, bounds.width - x)),
      height: Math.max(1, Math.min(activeRecipe.crop.height, bounds.height - y)),
    },
  });
}

function currentFrameDimensions(): { readonly width: number; readonly height: number } {
  if (selected === undefined) return { width: 1, height: 1 };
  if (!spriteEnabled) return { width: selected.width, height: selected.height };
  return {
    width: Math.max(1, Math.floor(selected.width / Math.max(1, columns))),
    height: Math.max(1, Math.floor(selected.height / Math.max(1, rows))),
  };
}

function numberValue(event: Event): number {
  return Number((event.currentTarget as HTMLInputElement).value);
}

async function save(): Promise<void> {
  if (selected === undefined || activeRecipe === null || busy) return;
  if (!nativeAvailable) {
    onnotify("info", "Creator changes can be saved in the desktop app.");
    return;
  }
  busy = true;
  try {
    const changes = await saveCreatorChanges({
      mediaId: selected.id,
      recipe: {
        crop: activeRecipe.crop,
        rotationDegrees: activeRecipe.rotationDegrees,
        flipHorizontal: activeRecipe.flipHorizontal,
        backgroundRemoval: activeRecipe.backgroundRemoval,
        brightness: activeRecipe.brightness,
        contrast: activeRecipe.contrast,
        saturation: activeRecipe.saturation,
      },
      spriteSequence: spriteEnabled
        ? {
            mediaId: selected.id,
            columns,
            rows,
            frameCount,
            frameDurationMs,
            playbackMode,
          }
        : null,
      clearSpriteSequence: !spriteEnabled && sequence !== null,
    });
    recipe = changes.recipe;
    sequence = changes.spriteSequence;
    onnotify("success", "Creator changes saved.");
    onreload();
  } catch (error) {
    onnotify(
      "error",
      error instanceof Error ? error.message : "Creator changes could not be saved.",
    );
  } finally {
    busy = false;
  }
}
</script>

<main class="creator-workspace">
  <header class="workspace-header">
    <div>
      <span class="eyebrow">CREATOR STUDIO</span>
      <h1>Shape media for your desktop</h1>
      <p>Adjust presentation without changing the imported source.</p>
    </div>
    <button class="save-button" type="button" disabled={selected === undefined || busy} onclick={() => void save()}>
      {#if busy}<span class="spin"><LoaderCircle size={17} /></span>{:else}<Save size={17} />{/if}
      Save changes
    </button>
  </header>

  {#if items.length === 0}
    <section class="empty-state">
      <span><Sparkles size={28} /></span>
      <h2>Import media to start creating</h2>
      <p>Your collection becomes the asset rail for non-destructive editing and sprite setup.</p>
    </section>
  {:else if selected !== undefined && activeRecipe !== null}
    <div class="studio-layout">
      <aside class="asset-rail" aria-label="Creator assets">
        <div class="panel-heading">
          <div><span>ASSETS</span><strong>{items.length}</strong></div>
        </div>
        <div class="asset-list">
          {#each items as item (item.id)}
            <button class:active={item.id === selected.id} type="button" onclick={() => onselect(item.id)}>
              <span class="asset-thumb">
                {#if assetUrls[item.id] !== undefined}<img src={assetUrls[item.id]} alt="" draggable="false" />{/if}
              </span>
              <span><strong title={item.displayName}>{item.displayName}</strong><small>{mediaFormatLabel(item.mediaFormat)}</small></span>
              {#if item.id === selected.id}<Check size={14} />{/if}
            </button>
          {/each}
        </div>
      </aside>

      <section class="preview-panel">
        <div class="preview-toolbar">
          <div>
            <strong>{selected.displayName}</strong>
            <span>{mediaDimensions(selected)} · {mediaFormatLabel(selected.mediaFormat)}</span>
          </div>
          <div class="tool-actions">
            <button type="button" title="Rotate left" aria-label="Rotate left" onclick={() => rotate(-1)}><RotateCcw size={17} /></button>
            <button type="button" title="Rotate right" aria-label="Rotate right" onclick={() => rotate(1)}><RotateCw size={17} /></button>
            <button class:active={activeRecipe.flipHorizontal} type="button" title="Flip horizontally" aria-label="Flip horizontally" onclick={() => patchRecipe({ flipHorizontal: !activeRecipe.flipHorizontal })}><FlipHorizontal2 size={17} /></button>
            <button type="button" title="Reset edits" aria-label="Reset edits" disabled={recipeIsDefault(activeRecipe)} onclick={resetRecipe}><Undo2 size={17} /></button>
          </div>
        </div>
        <div class="preview-stage">
          <EditedMediaPreview
            item={selected}
            src={assetUrls[selected.id]}
            recipe={activeRecipe}
            eager
            showCrop={activeRecipe.crop !== null}
            spriteColumns={spriteEnabled ? columns : 1}
            spriteRows={spriteEnabled ? rows : 1}
            nativeFrames={nativeFramePreview}
          />
        </div>
        {#if loadError !== null}<p class="inline-error">{loadError}</p>{/if}
      </section>

      <aside class="control-panel">
        <section>
          <div class="section-title"><Crop size={15} /><div><strong>Transform</strong><span>Crop, rotate, and mirror</span></div></div>
          <label class="check-row">
            <input type="checkbox" checked={activeRecipe.crop !== null} onchange={() => setFullCrop(activeRecipe.crop === null)} />
            <span><strong>Crop frame</strong><small>Start with a centered 80% crop.</small></span>
          </label>
          {#if activeRecipe.crop !== null}
            <div class="crop-grid">
              <label>X<input type="number" min="0" max={frameWidth - 1} value={activeRecipe.crop.x} onchange={(event) => { patchRecipe({ crop: { ...activeRecipe.crop!, x: numberValue(event) } }); clampCropToFrame(); }} /></label>
              <label>Y<input type="number" min="0" max={frameHeight - 1} value={activeRecipe.crop.y} onchange={(event) => { patchRecipe({ crop: { ...activeRecipe.crop!, y: numberValue(event) } }); clampCropToFrame(); }} /></label>
              <label>Width<input type="number" min="1" max={frameWidth} value={activeRecipe.crop.width} onchange={(event) => { patchRecipe({ crop: { ...activeRecipe.crop!, width: numberValue(event) } }); clampCropToFrame(); }} /></label>
              <label>Height<input type="number" min="1" max={frameHeight} value={activeRecipe.crop.height} onchange={(event) => { patchRecipe({ crop: { ...activeRecipe.crop!, height: numberValue(event) } }); clampCropToFrame(); }} /></label>
            </div>
          {/if}
        </section>

        <section>
          <div class="section-title"><Sparkles size={15} /><div><strong>Color</strong><span>Live presentation controls</span></div></div>
          <label class="range-row"><span>Brightness <output>{Math.round(activeRecipe.brightness * 100)}%</output></span><input type="range" min="0" max="2" step="0.05" value={activeRecipe.brightness} oninput={(event) => patchRecipe({ brightness: numberValue(event) })} /></label>
          <label class="range-row"><span>Contrast <output>{Math.round(activeRecipe.contrast * 100)}%</output></span><input type="range" min="0" max="2" step="0.05" value={activeRecipe.contrast} oninput={(event) => patchRecipe({ contrast: numberValue(event) })} /></label>
          <label class="range-row"><span>Saturation <output>{Math.round(activeRecipe.saturation * 100)}%</output></span><input type="range" min="0" max="2" step="0.05" value={activeRecipe.saturation} oninput={(event) => patchRecipe({ saturation: numberValue(event) })} /></label>
        </section>

        <section>
          <div class="section-title"><Sparkles size={15} /><div><strong>Background</strong><span>Transparent output modes</span></div></div>
          <div class="segmented four">
            {#each [["none", "Original"], ["edge", "Edge"], ["ai_fast", "AI fast"], ["ai_quality", "AI quality"]] as option}
              <button class:active={activeRecipe.backgroundRemoval === option[0]} type="button" onclick={() => patchRecipe({ backgroundRemoval: option[0] as BackgroundRemovalMode })}>{option[1]}</button>
            {/each}
          </div>
          {#if activeRecipe.backgroundRemoval.startsWith("ai_")}
            <p class="hint">Install the matching Fast or Quality model in Settings. Processing stays on this device.</p>
          {/if}
        </section>

        <section>
          <div class="section-title"><Grid3X3 size={15} /><div><strong>Sprite sheet</strong><span>Turn a still image into frames</span></div></div>
          <label class="check-row">
            <input type="checkbox" checked={spriteEnabled} disabled={!spriteEligible} onchange={(event) => setSpriteEnabled((event.currentTarget as HTMLInputElement).checked)} />
            <span><strong>Enable sprite playback</strong><small>Frames read left to right, top to bottom.</small></span>
          </label>
          {#if spriteEnabled}
            <div class="sprite-grid">
              <label>Columns<input type="number" min="1" max={Math.min(64, selected.width)} value={columns} onchange={(event) => setSpriteGrid("columns", numberValue(event))} /></label>
              <label>Rows<input type="number" min="1" max={Math.min(64, selected.height)} value={rows} onchange={(event) => setSpriteGrid("rows", numberValue(event))} /></label>
              <label>Frames<input type="number" min="1" max={spriteCapacity} bind:value={frameCount} /></label>
              <label>Frame ms<input type="number" min="10" max="60000" step="10" bind:value={frameDurationMs} /></label>
            </div>
            <div class="segmented">
              <button class:active={playbackMode === "loop"} type="button" onclick={() => (playbackMode = "loop")}>Loop</button>
              <button class:active={playbackMode === "once"} type="button" onclick={() => (playbackMode = "once")}>Once</button>
              <button class:active={playbackMode === "ping_pong"} type="button" onclick={() => (playbackMode = "ping_pong")}>Ping-pong</button>
            </div>
          {/if}
        </section>
      </aside>
    </div>
  {/if}
</main>

<style>
.creator-workspace { container: creator / inline-size; min-width: 0; min-height: 0; overflow: auto; background: var(--canvas-950); }
.workspace-header { display: flex; align-items: flex-end; justify-content: space-between; gap: 24px; padding: 28px 24px 20px; border-bottom: 1px solid var(--line-700); }
.eyebrow, .panel-heading span { color: var(--brand-300); font-size: 11px; font-weight: 750; letter-spacing: .09em; }
h1 { margin: 4px 0; font-family: "Space Grotesk Variable", sans-serif; font-size: 28px; }
.workspace-header p, .empty-state p { margin: 0; color: var(--ink-500); font-size: 13px; }
.save-button { display: inline-flex; min-height: 40px; align-items: center; gap: 8px; padding: 0 16px; color: var(--canvas-950); font-weight: 750; background: var(--brand-400); border: 1px solid var(--brand-400); border-radius: 6px; cursor: pointer; }
.save-button:hover { background: var(--brand-300); }
.save-button:disabled { cursor: wait; opacity: .55; }
.studio-layout { display: grid; grid-template-columns: 220px minmax(320px, 1fr) 340px; min-height: calc(100% - 105px); }
.asset-rail, .control-panel { min-width: 0; background: var(--canvas-900); }
.asset-rail { border-right: 1px solid var(--line-700); }
.control-panel { overflow: auto; border-left: 1px solid var(--line-700); }
.panel-heading { padding: 16px; border-bottom: 1px solid var(--line-700); }
.panel-heading div { display: flex; align-items: center; justify-content: space-between; }
.panel-heading strong { color: var(--ink-500); font-size: 12px; }
.asset-list { display: grid; gap: 4px; padding: 8px; }
.asset-list button { display: grid; grid-template-columns: 44px minmax(0, 1fr) 16px; align-items: center; gap: 10px; width: 100%; padding: 7px; color: var(--ink-300); text-align: left; background: transparent; border: 1px solid transparent; border-radius: 6px; cursor: pointer; }
.asset-list button:hover, .asset-list button.active { color: var(--ink-100); background: var(--surface-800); }
.asset-list button.active { border-color: var(--line-700); }
.asset-thumb { display: grid; width: 44px; height: 34px; overflow: hidden; place-items: center; background: var(--canvas-950); border-radius: 5px; }
.asset-thumb img { width: 100%; height: 100%; object-fit: contain; }
.asset-list button > span:nth-child(2) { display: grid; min-width: 0; gap: 3px; }
.asset-list strong, .asset-list small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.asset-list strong { font-size: 12px; }
.asset-list small { color: var(--ink-500); font-size: 11px; }
.preview-panel { display: grid; grid-template-rows: auto minmax(300px, 1fr) auto; min-width: 0; min-height: 0; }
.preview-toolbar { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 14px 16px; background: var(--surface-850); border-bottom: 1px solid var(--line-700); }
.preview-toolbar > div:first-child { display: grid; min-width: 0; gap: 3px; }
.preview-toolbar strong { overflow: hidden; font-size: 13px; text-overflow: ellipsis; white-space: nowrap; }
.preview-toolbar span { color: var(--ink-500); font-size: 11px; }
.tool-actions { display: flex; gap: 4px; }
.tool-actions button { display: grid; width: 36px; height: 36px; padding: 0; place-items: center; color: var(--ink-300); background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 6px; cursor: pointer; }
.tool-actions button:hover, .tool-actions button.active { color: var(--brand-300); background: var(--surface-750); }
.tool-actions button:disabled { opacity: .42; }
.preview-stage { display: grid; min-height: 0; padding: 28px; place-items: center; background: radial-gradient(circle at center, color-mix(in srgb, var(--brand-400) 9%, transparent), transparent 44%); }
.preview-stage :global(.edited-preview) { width: min(680px, 100%); box-shadow: 0 20px 60px rgb(0 0 0 / 28%); }
.inline-error { margin: 0; padding: 10px 16px; color: var(--danger-400); font-size: 12px; background: color-mix(in srgb, var(--danger-400) 8%, var(--surface-850)); border-top: 1px solid var(--line-700); }
.control-panel section { padding: 18px; border-bottom: 1px solid var(--line-700); }
.section-title { display: flex; align-items: flex-start; gap: 10px; margin-bottom: 15px; color: var(--brand-300); }
.section-title div { display: grid; gap: 2px; }
.section-title strong { color: var(--ink-100); font-size: 13px; }
.section-title span { color: var(--ink-500); font-size: 11px; }
.check-row { display: flex; align-items: center; gap: 10px; cursor: pointer; }
.check-row input { width: 18px; height: 18px; accent-color: var(--brand-400); }
.check-row span { display: grid; gap: 2px; }
.check-row strong { font-size: 12px; }
.check-row small, .hint { color: var(--ink-500); font-size: 11px; }
.crop-grid, .sprite-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-top: 14px; }
.crop-grid label, .sprite-grid label { display: grid; gap: 6px; color: var(--ink-500); font-size: 11px; }
input[type="number"] { width: 100%; height: 36px; padding: 0 9px; color: var(--ink-100); background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 6px; }
.range-row { display: grid; gap: 8px; margin-top: 12px; }
.range-row > span { display: flex; justify-content: space-between; color: var(--ink-300); font-size: 11px; }
.range-row input { width: 100%; accent-color: var(--brand-400); }
output { color: var(--ink-500); font-variant-numeric: tabular-nums; }
.segmented { display: grid; grid-template-columns: repeat(3, 1fr); gap: 4px; margin-top: 12px; }
.segmented.four { grid-template-columns: repeat(2, 1fr); }
.segmented button { min-height: 34px; padding: 0 8px; color: var(--ink-500); font-size: 11px; background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 5px; cursor: pointer; }
.segmented button:hover, .segmented button.active { color: var(--ink-100); background: var(--surface-750); }
.segmented button.active { border-color: color-mix(in srgb, var(--brand-400) 62%, var(--line-700)); }
.hint { margin: 10px 0 0; line-height: 1.5; }
.empty-state { display: grid; max-width: 480px; min-height: 60vh; margin: auto; padding: 48px; place-items: center; align-content: center; text-align: center; }
.empty-state > span { display: grid; width: 56px; height: 56px; margin-bottom: 12px; place-items: center; color: var(--brand-300); background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 12px; }
.empty-state h2 { margin: 0 0 8px; font-size: 20px; }
.spin { display: inline-flex; animation: spin 800ms linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@container creator (max-width: 1040px) { .studio-layout { grid-template-columns: 180px minmax(300px, 1fr); } .control-panel { grid-column: 1 / -1; display: grid; grid-template-columns: repeat(2, 1fr); border-top: 1px solid var(--line-700); border-left: 0; } }
@container creator (max-width: 680px) { .workspace-header { align-items: flex-start; flex-direction: column; } .studio-layout { grid-template-columns: 1fr; } .asset-rail { max-height: 180px; overflow: auto; border-right: 0; border-bottom: 1px solid var(--line-700); } .control-panel { display: block; } }
</style>
