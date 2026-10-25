import { afterEach, describe, expect, it, vi } from "vitest";
import type { FrameAsset } from "./contracts";
import { loadFrame } from "./overlay-frame-buffer";

afterEach(() => vi.unstubAllGlobals());

describe("overlay frame loading", () => {
  it("requests cross-origin frames with anonymous CORS before assigning the source", async () => {
    let crossOriginWhenSourceAssigned: string | null = null;

    class FakeImage {
      public decoding = "auto";
      public crossOrigin: string | null = null;
      public onload: (() => void) | null = null;
      public onerror: (() => void) | null = null;

      public set src(_value: string) {
        crossOriginWhenSourceAssigned = this.crossOrigin;
        queueMicrotask(() => this.onload?.());
      }
    }

    vi.stubGlobal("Image", FakeImage);
    const frame = {
      index: 0,
      durationMs: 80,
      path: "asset://localhost/frame.png",
    } satisfies FrameAsset;

    await loadFrame(frame);

    expect(crossOriginWhenSourceAssigned).toBe("anonymous");
  });
});
