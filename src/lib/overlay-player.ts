import type { FrameAsset, SpritePlaybackMode } from "./contracts";

export const FRAME_BATCH_SIZE = 32;
export const FRAME_PREFETCH_THRESHOLD = 4;
export const MIN_FRAME_DELAY_MS = 16;

export interface IndexedFrame {
  readonly index: number;
}

export interface BufferedFrameBatch<TFrame extends IndexedFrame> {
  readonly offset: number;
  readonly frames: readonly TFrame[];
}

export function nextFrameIndex(current: number, totalFrames: number): number {
  if (totalFrames <= 1) return 0;
  return current + 1 >= totalFrames ? 0 : current + 1;
}

export interface PlaybackStep {
  readonly index: number;
  readonly direction: 1 | -1;
  readonly finished: boolean;
}

export function nextPlaybackStep(
  current: number,
  totalFrames: number,
  mode: SpritePlaybackMode,
  direction: 1 | -1,
): PlaybackStep {
  if (totalFrames <= 1) return { index: 0, direction: 1, finished: mode === "once" };
  if (mode === "loop") {
    return { index: nextFrameIndex(current, totalFrames), direction: 1, finished: false };
  }
  if (mode === "once") {
    const finished = current >= totalFrames - 1;
    return {
      index: finished ? totalFrames - 1 : current + 1,
      direction: 1,
      finished,
    };
  }
  if (direction === 1 && current >= totalFrames - 1) {
    return { index: totalFrames - 2, direction: -1, finished: false };
  }
  if (direction === -1 && current <= 0) {
    return { index: 1, direction: 1, finished: false };
  }
  return { index: current + direction, direction, finished: false };
}

export function frameDelayMs(frame: Pick<FrameAsset, "durationMs">, speed: number): number {
  return Math.max(MIN_FRAME_DELAY_MS, Math.round(frame.durationMs / speed));
}

export function shouldSchedulePlayback(isPlaying: boolean, totalFrames: number): boolean {
  return isPlaying && totalFrames > 1;
}

export function batchOffsetForFrame(index: number): number {
  return Math.floor(index / FRAME_BATCH_SIZE) * FRAME_BATCH_SIZE;
}

export function prefetchBatchOffset(current: number, totalFrames: number): number | null {
  if (totalFrames <= FRAME_BATCH_SIZE) return null;
  const offset = batchOffsetForFrame(current);
  const end = Math.min(offset + FRAME_BATCH_SIZE, totalFrames);
  if (current < end - FRAME_PREFETCH_THRESHOLD) return null;
  return end >= totalFrames ? 0 : end;
}

export function appendBufferedBatch<TFrame extends IndexedFrame>(
  buffer: readonly BufferedFrameBatch<TFrame>[],
  batch: BufferedFrameBatch<TFrame>,
): readonly BufferedFrameBatch<TFrame>[] {
  const withoutPrevious = buffer.filter((candidate) => candidate.offset !== batch.offset);
  return [...withoutPrevious, batch].slice(-2);
}

export function findBufferedFrame<TFrame extends IndexedFrame>(
  buffer: readonly BufferedFrameBatch<TFrame>[],
  index: number,
): TFrame | undefined {
  for (const batch of buffer) {
    const frame = batch.frames.find((candidate) => candidate.index === index);
    if (frame !== undefined) return frame;
  }
  return undefined;
}
