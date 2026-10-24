<script lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onMount } from "svelte";
import { getFrameBatch, listOverlays } from "../api";
import { type FrameAsset, type OverlayId, type OverlayInstance, overlaySchema } from "../contracts";
import {
  appendBufferedBatch,
  type BufferedFrameBatch,
  batchOffsetForFrame,
  FRAME_BATCH_SIZE,
  findBufferedFrame,
  frameDelayMs,
  nextFrameIndex,
  prefetchBatchOffset,
  shouldSchedulePlayback,
} from "../overlay-player";

const OVERLAY_STATE_EVENT = "overlay:state";

interface LoadedFrame extends FrameAsset {
  readonly image: HTMLImageElement;
}

class OverlayPlaybackError extends Error {
  public constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "OverlayPlaybackError";
  }
}

export let overlayId: OverlayId;

let canvas: HTMLCanvasElement;
let overlay: OverlayInstance | null = null;
let buffer: readonly BufferedFrameBatch<LoadedFrame>[] = [];
let totalFrames = 0;
let currentIndex = 0;
let timer: number | undefined;
let unlisten: UnlistenFn | undefined;
let disposed = false;
let failed = false;
const loading = new Map<number, Promise<BufferedFrameBatch<LoadedFrame>>>();

$: opacity = overlay?.opacity ?? 0;
$: transform = overlay?.flipHorizontal ? "scaleX(-1)" : "scaleX(1)";

onMount(() => {
  void initialize().catch(handlePlaybackError);
  return () => {
    disposed = true;
    clearTimer();
    unlisten?.();
    loading.clear();
    buffer = [];
  };
});

async function initialize(): Promise<void> {
  const cleanup = await listen<unknown>(OVERLAY_STATE_EVENT, (event) => {
    const parsed = overlaySchema.safeParse(event.payload);
    if (parsed.success && parsed.data.id === overlayId) applyOverlay(parsed.data);
  });
  if (disposed) {
    cleanup();
    return;
  }
  unlisten = cleanup;

  const overlays = await listOverlays();
  const initial = overlays.find((candidate) => candidate.id === overlayId);
  if (initial === undefined) {
    throw new OverlayPlaybackError("Data overlay tidak ditemukan.");
  }
  overlay = initial;
  await ensureBatch(0);
  drawCurrentFrame();
  prefetch();
  scheduleNextFrame();
}

function applyOverlay(next: OverlayInstance): void {
  overlay = next;
  if (!next.isActive) {
    clearTimer();
    return;
  }
  scheduleNextFrame();
}

async function ensureBatch(offset: number): Promise<BufferedFrameBatch<LoadedFrame>> {
  const existing = buffer.find((batch) => batch.offset === offset);
  if (existing !== undefined) return existing;
  const inFlight = loading.get(offset);
  if (inFlight !== undefined) return inFlight;

  const request = loadBatch(offset);
  loading.set(offset, request);
  try {
    const loaded = await request;
    if (!disposed) buffer = appendBufferedBatch(buffer, loaded);
    return loaded;
  } finally {
    loading.delete(offset);
  }
}

async function loadBatch(offset: number): Promise<BufferedFrameBatch<LoadedFrame>> {
  const current = overlay;
  if (current === null) throw new OverlayPlaybackError("Overlay belum siap.");
  const batch = await getFrameBatch(current.mediaId, offset, FRAME_BATCH_SIZE);
  if (batch.frames.length === 0) {
    throw new OverlayPlaybackError("Batch frame kosong.");
  }
  totalFrames = batch.totalFrames;
  canvas.width = batch.width;
  canvas.height = batch.height;
  const frames = await Promise.all(batch.frames.map(loadFrame));
  return { offset, frames };
}

function loadFrame(frame: FrameAsset): Promise<LoadedFrame> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.decoding = "async";
    image.onload = () => resolve({ ...frame, image });
    image.onerror = () =>
      reject(new OverlayPlaybackError(`Frame ${frame.index + 1} tidak dapat dimuat.`));
    image.src = frame.path;
  });
}

function drawCurrentFrame(): void {
  const frame = findBufferedFrame(buffer, currentIndex);
  if (frame === undefined) {
    throw new OverlayPlaybackError(`Frame ${currentIndex + 1} belum tersedia.`);
  }
  const context = canvas.getContext("2d", { alpha: true });
  if (context === null) throw new OverlayPlaybackError("Canvas overlay tidak tersedia.");
  context.clearRect(0, 0, canvas.width, canvas.height);
  context.drawImage(frame.image, 0, 0, canvas.width, canvas.height);
}

function scheduleNextFrame(): void {
  clearTimer();
  const current = overlay;
  if (current === null || !shouldSchedulePlayback(current.isPlaying, totalFrames) || failed) return;
  const frame = findBufferedFrame(buffer, currentIndex);
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
  if (current === null || !shouldSchedulePlayback(current.isPlaying, totalFrames)) return;
  const next = nextFrameIndex(currentIndex, totalFrames);
  await ensureBatch(batchOffsetForFrame(next));
  if (disposed || overlay?.isPlaying !== true) return;
  currentIndex = next;
  drawCurrentFrame();
  prefetch();
  scheduleNextFrame();
}

function prefetch(): void {
  const offset = prefetchBatchOffset(currentIndex, totalFrames);
  if (offset !== null) void ensureBatch(offset).catch(handlePlaybackError);
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

<div class="overlay-stage" style:opacity style:transform aria-hidden="true">
  <canvas bind:this={canvas}></canvas>
</div>

<style>
.overlay-stage {
  width: 100vw;
  height: 100vh;
  transform-origin: center;
  background: transparent;
}

canvas {
  display: block;
  width: 100%;
  height: 100%;
}
</style>
