import { afterEach, describe, expect, it, vi } from "vitest";
import { getAiMaskBatch, getFrameBatch } from "./api";
import type { AiMaskBatch, FrameAsset, FrameBatch, MediaId } from "./contracts";
import { loadFrame, OverlayFrameBuffer } from "./overlay-frame-buffer";

vi.mock("./api", () => ({
  getAiMaskBatch: vi.fn(),
  getFrameBatch: vi.fn(),
}));

const MEDIA_ID = "2db87e5a-5968-4c30-89e9-bf1980e8bb5f" as MediaId;
const mockedGetFrameBatch = vi.mocked(getFrameBatch);
const mockedGetAiMaskBatch = vi.mocked(getAiMaskBatch);

afterEach(() => {
  vi.resetAllMocks();
  vi.unstubAllGlobals();
});

describe("overlay frame loading", () => {
  it("decodes an origin-clean bitmap before canvas pixel processing", async () => {
    const blob = new Blob(["frame"], { type: "image/png" });
    const bitmap = { width: 32, height: 24 };
    const fetchFrame = vi.fn(async () => new Response(blob, { status: 200 }));
    const decodeBitmap = vi.fn(async () => bitmap);
    vi.stubGlobal("fetch", fetchFrame);
    vi.stubGlobal("createImageBitmap", decodeBitmap);
    const frame = {
      index: 0,
      durationMs: 80,
      path: "asset://localhost/frame.png",
    } satisfies FrameAsset;

    const loaded = await loadFrame(frame);

    expect(fetchFrame).toHaveBeenCalledWith(frame.path);
    expect(decodeBitmap).toHaveBeenCalledWith(blob);
    expect(loaded.image).toBe(bitmap);
  });

  it("keeps fallback frames origin-clean when bitmap decoding fails", async () => {
    const images: FakeImage[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        throw new TypeError("Asset fetch blocked");
      }),
    );
    vi.stubGlobal("Image", fakeImageConstructor(images));
    const frame = {
      index: 0,
      durationMs: 80,
      path: "http://asset.localhost/frame.gif",
    } satisfies FrameAsset;

    await loadFrame(frame);

    expect(images).toHaveLength(1);
    expect(images[0]?.crossOrigin).toBe("anonymous");
  });

  it("ignores a stale frame request after invalidation", async () => {
    stubBitmapLoading();
    const stale = deferred<FrameBatch>();
    mockedGetFrameBatch
      .mockImplementationOnce(() => stale.promise)
      .mockResolvedValueOnce(frameBatch("frame-new.png", 64, 48));
    const canvas = { width: 0, height: 0 } as HTMLCanvasElement;
    const buffer = new OverlayFrameBuffer(canvas);

    const staleRequest = buffer.ensure(MEDIA_ID, 0);
    buffer.invalidate(MEDIA_ID);
    await buffer.ensure(MEDIA_ID, 0);
    stale.resolve(frameBatch("frame-old.png", 320, 240, 9));
    await staleRequest;

    expect(buffer.frame(0)?.path).toBe("frame-new.png");
    expect(buffer.totalFrames).toBe(1);
    expect({ width: canvas.width, height: canvas.height }).toEqual({ width: 64, height: 48 });
  });

  it("ignores stale masks that finish after invalidation", async () => {
    stubBitmapLoading();
    const staleMask = deferred<AiMaskBatch>();
    mockedGetFrameBatch
      .mockResolvedValueOnce(frameBatch("frame-old.png", 32, 24))
      .mockResolvedValueOnce(frameBatch("frame-new.png", 32, 24));
    mockedGetAiMaskBatch
      .mockImplementationOnce(() => staleMask.promise)
      .mockResolvedValueOnce(maskBatch("mask-new.png"));
    const buffer = new OverlayFrameBuffer({ width: 0, height: 0 } as HTMLCanvasElement);
    await buffer.ensure(MEDIA_ID, 0);

    const staleRequest = buffer.ensureMasks(MEDIA_ID, "fast", 0);
    await vi.waitFor(() => expect(mockedGetAiMaskBatch).toHaveBeenCalledTimes(1));
    buffer.invalidate(MEDIA_ID);
    await buffer.ensure(MEDIA_ID, 0);
    await buffer.ensureMasks(MEDIA_ID, "fast", 0);
    staleMask.resolve(maskBatch("mask-old.png"));
    await staleRequest;

    const mask = buffer.frame(0)?.aiMasks?.fast as (ImageBitmap & { label?: string }) | undefined;
    expect(buffer.frame(0)?.path).toBe("frame-new.png");
    expect(mask?.label).toBe("mask-new.png");
  });
});

function frameBatch(path: string, width: number, height: number, totalFrames = 1): FrameBatch {
  return {
    mediaId: MEDIA_ID,
    width,
    height,
    totalFrames,
    totalDurationMs: 80 * totalFrames,
    frames: [{ index: 0, durationMs: 80, path }],
  };
}

function maskBatch(path: string): AiMaskBatch {
  return { mediaId: MEDIA_ID, model: "fast", masks: [{ index: 0, path }] };
}

function deferred<T>(): { readonly promise: Promise<T>; readonly resolve: (value: T) => void } {
  let resolve = (_value: T): void => undefined;
  const promise = new Promise<T>((next) => {
    resolve = next;
  });
  return { promise, resolve };
}

function stubBitmapLoading(): void {
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: RequestInfo | URL) => new Response(String(input), { status: 200 })),
  );
  vi.stubGlobal(
    "createImageBitmap",
    vi.fn(async (blob: Blob) => ({ width: 32, height: 24, label: await blob.text() })),
  );
}

class FakeImage {
  public decoding = "";
  public crossOrigin = "";
  public onload: (() => void) | null = null;
  public onerror: (() => void) | null = null;

  public set src(_value: string) {
    queueMicrotask(() => this.onload?.());
  }
}

function fakeImageConstructor(images: FakeImage[]): typeof FakeImage {
  return class extends FakeImage {
    public constructor() {
      super();
      images.push(this);
    }
  };
}
