import { describe, expect, it } from "vitest";
import {
  appendBufferedBatch,
  frameDelayMs,
  nextFrameIndex,
  nextPlaybackStep,
  prefetchBatchOffset,
  shouldSchedulePlayback,
} from "./overlay-player";

describe("overlay playback timeline", () => {
  it("advances frames and loops to the beginning", () => {
    expect(nextFrameIndex(0, 3)).toBe(1);
    expect(nextFrameIndex(2, 3)).toBe(0);
  });

  it("uses original timing at normal speed", () => {
    expect(frameDelayMs({ durationMs: 80 }, 1)).toBe(80);
  });

  it("halves the delay at double speed", () => {
    expect(frameDelayMs({ durationMs: 80 }, 2)).toBe(40);
  });

  it("does not schedule paused or single-frame media", () => {
    expect(shouldSchedulePlayback(false, 8)).toBe(false);
    expect(shouldSchedulePlayback(true, 1)).toBe(false);
  });

  it("requests the following batch near a boundary", () => {
    expect(prefetchBatchOffset(28, 80)).toBe(32);
  });

  it("prefetches the first batch before the final batch loops", () => {
    expect(prefetchBatchOffset(76, 80)).toBe(0);
  });

  it("keeps only the two most recently used batches", () => {
    const first = { offset: 0, frames: [{ index: 0 }] };
    const second = { offset: 32, frames: [{ index: 32 }] };
    const third = { offset: 64, frames: [{ index: 64 }] };

    const result = appendBufferedBatch(appendBufferedBatch([first], second), third);

    expect(result.map((batch) => batch.offset)).toEqual([32, 64]);
  });
});

describe("nextPlaybackStep", () => {
  it("stops once playback on the final frame", () => {
    expect(nextPlaybackStep(1, 3, "once", 1)).toEqual({
      index: 2,
      direction: 1,
      finished: false,
    });
    expect(nextPlaybackStep(2, 3, "once", 1)).toEqual({
      index: 2,
      direction: 1,
      finished: true,
    });
  });

  it("reverses ping-pong playback at both edges", () => {
    expect(nextPlaybackStep(2, 3, "ping_pong", 1)).toEqual({
      index: 1,
      direction: -1,
      finished: false,
    });
    expect(nextPlaybackStep(0, 3, "ping_pong", -1)).toEqual({
      index: 1,
      direction: 1,
      finished: false,
    });
  });
});
