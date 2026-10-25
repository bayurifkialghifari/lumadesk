<script lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { currentMonitor, getCurrentWindow, type PhysicalPosition } from "@tauri-apps/api/window";
import { onMount } from "svelte";
import { duplicateOverlay, listOverlays, moveOverlay, removeOverlay, updateOverlay } from "../api";
import { type OverlayId, type OverlayInstance, overlaySchema } from "../contracts";
import { OverlayFrameBuffer, OverlayPlaybackError } from "../overlay-frame-buffer";
import {
  batchOffsetForFrame,
  frameDelayMs,
  nextFrameIndex,
  prefetchBatchOffset,
  shouldSchedulePlayback,
} from "../overlay-player";
import { LatestPositionSaver } from "../overlay-position-saver";
import { userErrorMessage } from "../presentation";
import OverlayStage from "./OverlayStage.svelte";

const OVERLAY_STATE_EVENT = "overlay:state";
export let overlayId: OverlayId;

let canvas: HTMLCanvasElement;
let overlay: OverlayInstance | null = null;
let frameBuffer: OverlayFrameBuffer | null = null;
let currentIndex = 0;
let timer: number | undefined;
let unlisten: readonly UnlistenFn[] = [];
let disposed = false;
let failed = false;
let overlayRevision = 0;
let actionBusy = false;
let actionError: string | null = null;
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
  const frames = requireFrameBuffer();
  await frames.ensure(overlay.mediaId, 0);
  frames.draw(currentIndex);
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
    overlay = await updateOverlay(current.id, patch);
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
  overlay = next;
  if (!next.isActive) {
    clearTimer();
    return;
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
  const next = nextFrameIndex(currentIndex, frames.totalFrames);
  await frames.ensure(current.mediaId, batchOffsetForFrame(next));
  if (disposed || overlay?.isPlaying !== true) return;
  currentIndex = next;
  frames.draw(currentIndex);
  prefetch();
  scheduleNextFrame();
}

function prefetch(): void {
  const current = overlay;
  const frames = frameBuffer;
  if (current === null || frames === null) return;
  const offset = prefetchBatchOffset(currentIndex, frames.totalFrames);
  if (offset !== null) void frames.ensure(current.mediaId, offset).catch(handlePlaybackError);
}

function requireFrameBuffer(): OverlayFrameBuffer {
  if (frameBuffer === null) throw new OverlayPlaybackError("The overlay buffer is not ready.");
  return frameBuffer;
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
