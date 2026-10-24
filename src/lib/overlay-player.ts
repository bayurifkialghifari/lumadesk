import type { FrameAsset } from "./contracts";

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
