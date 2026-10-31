<script lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { currentMonitor, getCurrentWindow, type PhysicalPosition } from "@tauri-apps/api/window";
import { onMount } from "svelte";
import {
  duplicateOverlay,
  getEditRecipe,
  getSpriteSequence,
  listOverlays,
  moveOverlay,
  removeOverlay,
  updateOverlay,
} from "../api";
import {
  type AiModel,
  type EditRecipe,
  editRecipeSchema,
  type OverlayId,
  type OverlayInstance,
  overlaySchema,
  type SpriteSequence,
} from "../contracts";
import { defaultEditRecipe } from "../edit-recipes";
import { OverlayFrameBuffer, OverlayPlaybackError } from "../overlay-frame-buffer";
import {
  batchOffsetForFrame,
  frameDelayMs,
  nextPlaybackStep,
  prefetchBatchOffset,
  shouldSchedulePlayback,
} from "../overlay-player";
import { LatestPositionSaver } from "../overlay-position-saver";
import { userErrorMessage } from "../presentation";
import OverlayStage from "./OverlayStage.svelte";

const OVERLAY_STATE_EVENT = "overlay:state";
const MEDIA_EDIT_RECIPE_EVENT = "media:edit-recipe-changed";
const MEDIA_SPRITE_SEQUENCE_EVENT = "media:sprite-sequence-changed";
export let overlayId: OverlayId;

let canvas: HTMLCanvasElement;
let overlay: OverlayInstance | null = null;
let recipe: EditRecipe | null = null;
let spriteSequence: SpriteSequence | null = null;
let frameBuffer: OverlayFrameBuffer | null = null;
let currentIndex = 0;
let playbackDirection: 1 | -1 = 1;
let playbackFinished = false;
let timer: number | undefined;
let unlisten: readonly UnlistenFn[] = [];
let disposed = false;
let failed = false;
let overlayRevision = 0;
let actionBusy = false;
let actionError: string | null = null;
let backgroundRemovalFailed = false;
let unavailableAiModel: AiModel | null = null;
const nativeWindow = getCurrentWindow();
const positionSaver = new LatestPositionSaver<PhysicalPosition>(
  async (position) => {
    const scaleFactor = await nativeWindow.scaleFactor();
    const logical = position.toLogical(scaleFactor);
    const monitor = await currentMonitor();
    await moveOverlay(overlayId, monitor?.name ?? null, logical.x, logical.y);
  },
  (error) => showActionError(error, "The overlay position could not be saved."),
);

$: mediaOpacity = overlay?.opacity ?? 0;
$: mediaTransform = overlay?.flipHorizontal ? "scaleX(-1)" : "scaleX(1)";

onMount(() => {
  frameBuffer = new OverlayFrameBuffer(canvas);
  void initialize().catch(handlePlaybackError);
  return () => {
    disposed = true;
    clearTimer();
    positionSaver.dispose();
    for (const cleanup of unlisten) cleanup();
    frameBuffer?.clear();
    frameBuffer = null;
  };
});

async function initialize(): Promise<void> {
  const cleanups = await Promise.all([
    listen<unknown>(OVERLAY_STATE_EVENT, (event) => {
      const parsed = overlaySchema.safeParse(event.payload);
      if (parsed.success && parsed.data.id === overlayId) {
        overlayRevision += 1;
        applyOverlay(parsed.data);
      }
    }),
    listen<unknown>(MEDIA_EDIT_RECIPE_EVENT, (event) => {
      const parsed = editRecipeSchema.safeParse(event.payload);
      if (parsed.success && parsed.data.mediaId === overlay?.mediaId) {
        recipe = parsed.data;
        backgroundRemovalFailed = false;
        unavailableAiModel = null;
        const frames = frameBuffer;
        const current = overlay;
        if (frames?.frame(currentIndex) !== undefined && current !== null) {
          drawCurrentFrame(frames, current);
          void prepareAiMask(currentIndex);
        }
      }
    }),
    listen<string>(MEDIA_SPRITE_SEQUENCE_EVENT, (event) => {
      if (event.payload === overlay?.mediaId) void reloadMedia().catch(handlePlaybackError);
    }),
    nativeWindow.onMoved((event) => {
      if (disposed) return;
      positionSaver.schedule(event.payload);
    }),
  ]);
  if (disposed) {
    for (const cleanup of cleanups) cleanup();
    return;
  }
  unlisten = cleanups;

  const overlayRevisionAtRequest = overlayRevision;
  const overlays = await listOverlays();
  if (disposed) return;
  const initial = overlays.find((candidate) => candidate.id === overlayId);
  if (overlayRevision === overlayRevisionAtRequest && initial !== undefined) overlay = initial;
  if (overlay === null) {
    throw new OverlayPlaybackError("Overlay data was not found.");
  }
  [recipe, spriteSequence] = await Promise.all([
    getEditRecipe(overlay.mediaId),
    getSpriteSequence(overlay.mediaId),
  ]);
  const frames = requireFrameBuffer();
  await frames.ensure(overlay.mediaId, 0);
  await prepareAiMask(0);
  drawCurrentFrame(frames, overlay);
  prefetch();
  scheduleNextFrame();
}

async function reloadMedia(): Promise<void> {
  const current = overlay;
  const frames = frameBuffer;
  if (current === null || frames === null) return;
  clearTimer();
  failed = false;
  currentIndex = 0;
  playbackDirection = 1;
  playbackFinished = false;
  backgroundRemovalFailed = false;
  spriteSequence = await getSpriteSequence(current.mediaId);
  unavailableAiModel = null;
  frames.invalidate(current.mediaId);
  await frames.ensure(current.mediaId, 0);
  await prepareAiMask(0);
  if (disposed) return;
  drawCurrentFrame(frames, current);
  prefetch();
  scheduleNextFrame();
}

async function startDrag(event: MouseEvent): Promise<void> {
  if (event.button !== 0 || actionBusy) return;
  event.preventDefault();
  try {
    await nativeWindow.startDragging();
  } catch (error) {
    showActionError(error, "The overlay could not be moved.");
  }
}

async function patchOverlay(patch: Parameters<typeof updateOverlay>[1]): Promise<void> {
  const current = overlay;
  if (current === null || actionBusy) return;
  actionBusy = true;
  actionError = null;
  try {
    applyOverlay(await updateOverlay(current.id, patch));
  } catch (error) {
    showActionError(error, "The overlay controls could not be updated.");
  } finally {
    actionBusy = false;
  }
}

async function duplicateCurrent(): Promise<void> {
  const current = overlay;
  if (current === null || actionBusy) return;
  actionBusy = true;
  actionError = null;
  try {
    await duplicateOverlay(current.id);
  } catch (error) {
    showActionError(error, "The overlay could not be duplicated.");
  } finally {
    actionBusy = false;
  }
}

async function closeCurrent(): Promise<void> {
  const current = overlay;
  if (current === null || actionBusy) return;
  actionBusy = true;
  actionError = null;
  try {
    await removeOverlay(current.id);
  } catch (error) {
    actionBusy = false;
    showActionError(error, "The overlay could not be closed.");
  }
}

function showActionError(error: unknown, fallback: string): void {
  actionError = error instanceof Error ? userErrorMessage(error, fallback) : fallback;
}

function applyOverlay(next: OverlayInstance): void {
  if (next.removeBackground && overlay?.removeBackground !== true) {
    backgroundRemovalFailed = false;
  }
  if (!next.removeBackground) backgroundRemovalFailed = false;
  overlay = next;
  if (!next.isActive) {
    clearTimer();
    return;
  }
  const frames = frameBuffer;
  if (frames?.frame(currentIndex) !== undefined) {
    drawCurrentFrame(frames, next);
  }
  scheduleNextFrame();
}

function scheduleNextFrame(): void {
  clearTimer();
  const current = overlay;
  const frames = frameBuffer;
  if (
    current === null ||
    frames === null ||
    !shouldSchedulePlayback(current.isPlaying, frames.totalFrames) ||
    playbackFinished ||
    failed
  )
    return;
  const frame = frames.frame(currentIndex);
  if (frame === undefined) return;
  timer = window.setTimeout(
    () => {
      timer = undefined;
      void advanceFrame().catch(handlePlaybackError);
    },
    frameDelayMs(frame, current.speed),
  );
}

async function advanceFrame(): Promise<void> {
  const current = overlay;
  const frames = frameBuffer;
  if (
    current === null ||
    frames === null ||
    !shouldSchedulePlayback(current.isPlaying, frames.totalFrames)
  )
    return;
  const step = nextPlaybackStep(
    currentIndex,
    frames.totalFrames,
    spriteSequence?.playbackMode ?? "loop",
    playbackDirection,
  );
  playbackDirection = step.direction;
  playbackFinished = step.finished;
  if (step.finished) {
    clearTimer();
    return;
  }
  await frames.ensure(current.mediaId, batchOffsetForFrame(step.index));
  await prepareAiMask(step.index);
  const latest = overlay;
  if (disposed || latest?.isPlaying !== true) return;
  currentIndex = step.index;
  drawCurrentFrame(frames, latest);
  prefetch();
  scheduleNextFrame();
}

function prefetch(): void {
  const current = overlay;
  const frames = frameBuffer;
  if (current === null || frames === null) return;
  const offset = prefetchBatchOffset(currentIndex, frames.totalFrames);
  if (offset !== null) {
    void frames.ensure(current.mediaId, offset).catch(handlePlaybackError);
    const model = activeAiModel();
    if (model !== null && unavailableAiModel !== model) {
      void frames.ensureMasks(current.mediaId, model, offset).catch((error) => {
        unavailableAiModel = model;
        showActionError(error, "The local AI mask is unavailable; edge removal is being used.");
      });
    }
  }
}

async function prepareAiMask(index: number): Promise<void> {
  const current = overlay;
  const frames = frameBuffer;
  const model = activeAiModel();
  if (current === null || frames === null || model === null || unavailableAiModel === model) return;
  try {
    await frames.ensureMasks(current.mediaId, model, batchOffsetForFrame(index));
    if (
      !disposed &&
      currentIndex === index &&
      overlay?.mediaId === current.mediaId &&
      activeAiModel() === model
    ) {
      drawCurrentFrame(frames, overlay);
    }
  } catch (error) {
    unavailableAiModel = model;
    showActionError(error, "The local AI mask is unavailable; edge removal is being used.");
  }
}

function activeAiModel(): AiModel | null {
  if (recipe?.backgroundRemoval === "ai_fast") return "fast";
  if (recipe?.backgroundRemoval === "ai_quality") return "quality";
  return null;
}

function requireFrameBuffer(): OverlayFrameBuffer {
  if (frameBuffer === null) throw new OverlayPlaybackError("The overlay buffer is not ready.");
  return frameBuffer;
}

function drawCurrentFrame(frames: OverlayFrameBuffer, current: OverlayInstance): void {
  const activeRecipe = recipe ?? defaultEditRecipe(current.mediaId);
  const drawableRecipe = backgroundRemovalFailed
    ? { ...activeRecipe, backgroundRemoval: "none" as const }
    : activeRecipe;
  try {
    frames.draw(currentIndex, drawableRecipe, current.removeBackground && !backgroundRemovalFailed);
  } catch (error) {
    if (!current.removeBackground && drawableRecipe.backgroundRemoval === "none") throw error;
    backgroundRemovalFailed = true;
    showActionError(error, "The background could not be removed from this media.");
    frames.draw(currentIndex, { ...activeRecipe, backgroundRemoval: "none" }, false);
  }
}

function clearTimer(): void {
  if (timer !== undefined) window.clearTimeout(timer);
  timer = undefined;
}

function handlePlaybackError(error: unknown): void {
  failed = true;
  clearTimer();
  if (error instanceof Error) {
    console.error("LumaDesk overlay playback failed", error);
  } else {
    console.error("LumaDesk overlay playback failed with an unknown error");
  }
}
</script>

<svelte:head><title>LumaDesk Overlay</title></svelte:head>

<OverlayStage
  bind:canvas
  {mediaOpacity}
  {mediaTransform}
  {overlay}
  busy={actionBusy}
  error={actionError}
  onstartdrag={(event) => void startDrag(event)}
  onpatch={(patch) => void patchOverlay(patch)}
  onduplicate={() => void duplicateCurrent()}
  onclose={() => void closeCurrent()}
/>
