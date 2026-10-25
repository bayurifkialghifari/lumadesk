import { afterEach, describe, expect, it, vi } from "vitest";
import { OVERLAY_CONTROLS_IDLE_MS, OverlayControlsVisibility } from "./overlay-controls-visibility";

afterEach(() => vi.useRealTimers());

describe("overlay controls visibility", () => {
  it("reveals controls and hides them after the idle delay", () => {
    vi.useFakeTimers();
    const states: boolean[] = [];
    const visibility = new OverlayControlsVisibility((visible) => states.push(visible));

    visibility.reveal();
    vi.advanceTimersByTime(OVERLAY_CONTROLS_IDLE_MS - 1);
    expect(states).toEqual([true]);

    vi.advanceTimersByTime(1);
    expect(states).toEqual([true, false]);
  });

  it("restarts the idle delay when the toolbar is used", () => {
    vi.useFakeTimers();
    const states: boolean[] = [];
    const visibility = new OverlayControlsVisibility((visible) => states.push(visible));

    visibility.reveal();
    vi.advanceTimersByTime(2_000);
    visibility.activity();
    vi.advanceTimersByTime(2_999);
    expect(states).toEqual([true]);

    vi.advanceTimersByTime(1);
    expect(states).toEqual([true, false]);
  });

  it("stays visible while pointer or focus interaction holds the toolbar", () => {
    vi.useFakeTimers();
    const states: boolean[] = [];
    const visibility = new OverlayControlsVisibility((visible) => states.push(visible));

    visibility.reveal();
    visibility.hold("pointer");
    visibility.hold("focus");
    vi.advanceTimersByTime(10_000);
    visibility.release("pointer");
    vi.advanceTimersByTime(10_000);
    expect(states).toEqual([true]);

    visibility.release("focus");
    vi.advanceTimersByTime(OVERLAY_CONTROLS_IDLE_MS);
    expect(states).toEqual([true, false]);
  });
});
