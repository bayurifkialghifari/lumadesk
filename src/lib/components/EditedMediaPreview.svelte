<script lang="ts">
import { Image as ImageIcon } from "@lucide/svelte";
import { onMount } from "svelte";
import type { AiModel, EditRecipe, MediaItem } from "../contracts";
import { editFilter, editTransform } from "../edit-recipes";
import { OverlayFrameBuffer } from "../overlay-frame-buffer";

export let item: MediaItem;
export let src: string | undefined = undefined;
export let recipe: EditRecipe;
export let eager = false;
export let showCrop = false;
export let spriteColumns = 1;
export let spriteRows = 1;
export let nativeFrames = false;

let failed = false;
let canvas: HTMLCanvasElement | undefined;
let frameBuffer: OverlayFrameBuffer | null = null;
let mounted = false;
let renderedMediaId: string | null = null;
let scheduledRender = "";
let nativeReady = false;
let nativeFailed = false;
let aiFallback = false;
const unavailableModels = new Set<AiModel>();
$: if (src !== undefined) failed = false;
$: crop = recipe.crop;
$: previewWidth = Math.max(1, Math.floor(item.width / Math.max(1, spriteColumns)));
$: previewHeight = Math.max(1, Math.floor(item.height / Math.max(1, spriteRows)));
$: spritePreview = spriteColumns > 1 || spriteRows > 1;
$: clip =
  crop === null
    ? "none"
    : "inset(" +
      Math.max(0, (crop.y / previewHeight) * 100) +
      "% " +
      Math.max(0, ((previewWidth - crop.x - crop.width) / previewWidth) * 100) +
      "% " +
      Math.max(0, ((previewHeight - crop.y - crop.height) / previewHeight) * 100) +
      "% " +
      Math.max(0, (crop.x / previewWidth) * 100) +
      "%)";
$: nativeRenderKey = nativeFrames ? JSON.stringify({ mediaId: item.id, recipe }) : "";
$: if (mounted && canvas !== undefined && nativeRenderKey !== scheduledRender) {
  scheduledRender = nativeRenderKey;
  if (nativeFrames) void renderNativeFrame(nativeRenderKey);
  else {
    frameBuffer?.invalidate();
    renderedMediaId = null;
    nativeReady = false;
    nativeFailed = false;
    aiFallback = false;
  }
}

onMount(() => {
  mounted = true;
  if (canvas !== undefined) frameBuffer = new OverlayFrameBuffer(canvas);
  scheduledRender = "";
  return () => {
    mounted = false;
    frameBuffer?.clear();
    frameBuffer = null;
  };
});

async function renderNativeFrame(requestKey: string): Promise<void> {
  const frames = frameBuffer;
  if (frames === null) return;
  const mediaId = item.id;
  const activeRecipe = recipe;
  if (renderedMediaId !== mediaId) {
    frames.invalidate();
    renderedMediaId = mediaId;
    nativeReady = false;
    unavailableModels.clear();
  }
  nativeFailed = false;
  aiFallback = false;
  try {
    await frames.ensure(mediaId, 0);
    if (!requestIsCurrent(requestKey, mediaId)) return;
    frames.draw(0, activeRecipe, false);
    nativeReady = true;
    const model = aiModelFor(activeRecipe);
    if (model === null) return;
    if (unavailableModels.has(model)) {
      aiFallback = true;
      return;
    }
    try {
      await frames.ensureMasks(mediaId, model, 0);
      if (!requestIsCurrent(requestKey, mediaId)) return;
      frames.draw(0, activeRecipe, false);
    } catch {
      unavailableModels.add(model);
      if (requestIsCurrent(requestKey, mediaId)) aiFallback = true;
    }
  } catch {
    if (!requestIsCurrent(requestKey, mediaId)) return;
    nativeReady = false;
    nativeFailed = true;
  }
}

function requestIsCurrent(requestKey: string, mediaId: string): boolean {
  return mounted && nativeFrames && nativeRenderKey === requestKey && item.id === mediaId;
}

function aiModelFor(activeRecipe: EditRecipe): AiModel | null {
  if (activeRecipe.backgroundRemoval === "ai_fast") return "fast";
  if (activeRecipe.backgroundRemoval === "ai_quality") return "quality";
  return null;
}
</script>

<div class="edited-preview" class:cropOutline={showCrop}>
  <canvas
    bind:this={canvas}
    class="native-frame"
    class:ready={nativeReady}
    aria-label={item.displayName + " edited preview"}
  >{item.displayName} edited preview</canvas>
  {#if (!nativeFrames || !nativeReady) && src !== undefined && !failed}
    <div
      class="media-layer"
      role="img"
      aria-label={item.displayName + " edited preview"}
      style:filter={editFilter(recipe)}
      style:transform={editTransform(recipe)}
      style:clip-path={clip}
    >
      {#if spritePreview}
        <span class="sprite-cell">
          <img
            class="sprite-source"
            src={src}
            alt=""
            loading={eager ? "eager" : "lazy"}
            decoding="async"
            draggable="false"
            style:width={`${spriteColumns * 100}%`}
            style:height={`${spriteRows * 100}%`}
            onerror={() => (failed = true)}
          />
        </span>
      {:else}
        <img
          class="standard-image"
          src={src}
          alt=""
          width={item.width}
          height={item.height}
          loading={eager ? "eager" : "lazy"}
          decoding="async"
          draggable="false"
          onerror={() => (failed = true)}
        />
      {/if}
    </div>
  {:else if !nativeFrames || nativeFailed}
    <div class="placeholder" aria-label="Preview unavailable">
      <ImageIcon size={28} strokeWidth={1.7} />
    </div>
  {/if}
  {#if recipe.backgroundRemoval !== "none"}
    <span class="effect-badge" title={aiFallback ? "The local AI model is unavailable, so this preview uses the edge mask." : undefined}>
      {recipe.backgroundRemoval === "edge" ? "Edge mask" : recipe.backgroundRemoval === "ai_fast" ? "AI fast" : "AI quality"}
      {#if aiFallback} · edge preview{/if}
    </span>
  {/if}
</div>

<style>
.edited-preview {
  position: relative;
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
  background-position: 0 0, 0 10px, 10px -10px, -10px 0;
  background-size: 20px 20px;
  border-radius: 8px;
}

.media-layer {
  display: block;
  width: 100%;
  height: 100%;
  transform-origin: center;
  transition: filter 140ms ease, transform 140ms ease, clip-path 140ms ease;
}

.native-frame {
  display: none;
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.native-frame.ready { display: block; }

.standard-image { display: block; width: 100%; height: 100%; object-fit: contain; }
.sprite-cell { position: relative; display: block; width: 100%; height: 100%; overflow: hidden; }
.sprite-source { position: absolute; top: 0; left: 0; display: block; max-width: none; object-fit: fill; }

.cropOutline::after {
  position: absolute;
  inset: 12px;
  pointer-events: none;
  border: 1px dashed color-mix(in srgb, var(--brand-300) 76%, transparent);
  content: "";
}

.placeholder { display: grid; width: 100%; height: 100%; place-items: center; color: var(--ink-500); }
.effect-badge {
  position: absolute;
  right: 10px;
  bottom: 10px;
  padding: 5px 8px;
  color: var(--ink-100);
  font-size: 11px;
  font-weight: 700;
  background: color-mix(in srgb, var(--canvas-950) 82%, transparent);
  border: 1px solid var(--line-700);
  border-radius: 999px;
}
</style>
