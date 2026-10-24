import { afterEach, describe, expect, it, vi } from "vitest";
import { LatestPositionSaver } from "./overlay-position-saver";

afterEach(() => vi.useRealTimers());

describe("latest overlay position saver", () => {
  it("debounces movement and serializes the newest pending position", async () => {
    vi.useFakeTimers();
    const saved: number[] = [];
    let releaseFirst: (() => void) | undefined;
    const saver = new LatestPositionSaver<number>(
      async (position) => {
        saved.push(position);
        if (position === 1) {
          await new Promise<void>((resolve) => {
            releaseFirst = resolve;
          });
        }
      },
      vi.fn(),
      10,
    );

    saver.schedule(0);
    saver.schedule(1);
    await vi.advanceTimersByTimeAsync(10);
    saver.schedule(2);
    saver.schedule(3);
    await vi.advanceTimersByTimeAsync(10);
    expect(saved).toEqual([1]);

    releaseFirst?.();
    await vi.waitFor(() => expect(saved).toEqual([1, 3]));
    saver.dispose();
  });
});
