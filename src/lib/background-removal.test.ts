import { describe, expect, it } from "vitest";
import { removeEdgeConnectedBackground } from "./background-removal";

function solidFrame(
  width: number,
  height: number,
  color: readonly [number, number, number, number],
): Uint8ClampedArray {
  const pixels = new Uint8ClampedArray(width * height * 4);
  for (let index = 0; index < width * height; index += 1) {
    pixels.set(color, index * 4);
  }
  return pixels;
}

function setPixel(
  pixels: Uint8ClampedArray,
  width: number,
  x: number,
  y: number,
  color: readonly [number, number, number, number],
): void {
  pixels.set(color, (y * width + x) * 4);
}

function alphaAt(pixels: Uint8ClampedArray, width: number, x: number, y: number): number {
  return pixels[(y * width + x) * 4 + 3] ?? 0;
}

describe("edge-connected background removal", () => {
  it("removes a solid edge background while preserving the foreground", () => {
    const pixels = solidFrame(3, 3, [245, 245, 245, 255]);
    setPixel(pixels, 3, 1, 1, [210, 30, 40, 255]);

    removeEdgeConnectedBackground(pixels, 3, 3);

    expect(alphaAt(pixels, 3, 0, 0)).toBe(0);
    expect(alphaAt(pixels, 3, 1, 1)).toBe(255);
  });

  it("preserves matching colors enclosed inside the foreground", () => {
    const pixels = solidFrame(5, 5, [250, 250, 250, 255]);
    for (let position = 1; position < 4; position += 1) {
      setPixel(pixels, 5, position, 1, [20, 20, 20, 255]);
      setPixel(pixels, 5, position, 3, [20, 20, 20, 255]);
      setPixel(pixels, 5, 1, position, [20, 20, 20, 255]);
      setPixel(pixels, 5, 3, position, [20, 20, 20, 255]);
    }

    removeEdgeConnectedBackground(pixels, 5, 5);

    expect(alphaAt(pixels, 5, 0, 0)).toBe(0);
    expect(alphaAt(pixels, 5, 2, 2)).toBe(255);
  });

  it("keeps existing transparent pixels transparent", () => {
    const pixels = solidFrame(2, 2, [80, 120, 160, 255]);
    setPixel(pixels, 2, 0, 0, [0, 0, 0, 0]);

    removeEdgeConnectedBackground(pixels, 2, 2);

    expect(alphaAt(pixels, 2, 0, 0)).toBe(0);
  });
});
