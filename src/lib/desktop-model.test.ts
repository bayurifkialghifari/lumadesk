import { describe, expect, it } from "vitest";
import {
  normalizeOverlaySpeed,
  normalizeOverlayWidth,
  opacityFromPercent,
  overlayHeight,
} from "./desktop-model";

const item = {
  overlay: { width: 320 },
  media: { width: 640, height: 360 },
};

describe("desktop overlay controls", () => {
  it("keeps the original aspect ratio when reporting overlay height", () => {
    expect(overlayHeight(item)).toBe(180);
  });

  it("normalizes values to the limits exposed by the desktop manager", () => {
    expect(normalizeOverlayWidth(12)).toBe(64);
    expect(normalizeOverlayWidth(3_000)).toBe(2_048);
    expect(opacityFromPercent(4)).toBe(0.1);
    expect(opacityFromPercent(82)).toBe(0.82);
    expect(normalizeOverlaySpeed(8)).toBe(4);
  });
});
