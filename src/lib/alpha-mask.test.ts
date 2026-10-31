import { describe, expect, it } from "vitest";
import { applyAlphaMask } from "./alpha-mask";

describe("AI alpha masks", () => {
  it("multiplies source alpha by the grayscale mask", () => {
    const source = new Uint8ClampedArray([20, 40, 60, 255, 80, 100, 120, 128]);
    const mask = new Uint8ClampedArray([255, 255, 255, 255, 64, 64, 64, 255]);

    applyAlphaMask(source, mask);

    expect([...source]).toEqual([20, 40, 60, 255, 80, 100, 120, 32]);
  });

  it("rejects a mask that cannot cover the source frame", () => {
    expect(() => applyAlphaMask(new Uint8ClampedArray(8), new Uint8ClampedArray(4))).toThrow(
      RangeError,
    );
  });
});
