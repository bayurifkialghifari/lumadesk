import { getFrameBatch } from "./api";
import type { FrameAsset, MediaId } from "./contracts";
import {
  appendBufferedBatch,
  type BufferedFrameBatch,
  FRAME_BATCH_SIZE,
  findBufferedFrame,
} from "./overlay-player";

interface LoadedFrame extends FrameAsset {
  readonly image: HTMLImageElement;
}

export class OverlayPlaybackError extends Error {
  public constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "OverlayPlaybackError";
  }
}

export class OverlayFrameBuffer {
  readonly #canvas: HTMLCanvasElement;
  readonly #loading = new Map<string, Promise<BufferedFrameBatch<LoadedFrame>>>();
  #batches: readonly BufferedFrameBatch<LoadedFrame>[] = [];
  #mediaId: MediaId | null = null;
  #totalFrames = 0;
  #disposed = false;

  public constructor(canvas: HTMLCanvasElement) {
    this.#canvas = canvas;
  }

  public get totalFrames(): number {
    return this.#totalFrames;
  }

  public frame(index: number): LoadedFrame | undefined {
    return findBufferedFrame(this.#batches, index);
  }

  public async ensure(mediaId: MediaId, offset: number): Promise<void> {
    this.#selectMedia(mediaId);
    if (this.#batches.some((batch) => batch.offset === offset)) return;
    const requestKey = `${mediaId}:${offset}`;
    const inFlight = this.#loading.get(requestKey);
    if (inFlight !== undefined) {
      await inFlight;
      return;
    }

    const request = this.#loadBatch(mediaId, offset);
    this.#loading.set(requestKey, request);
    try {
      const loaded = await request;
      if (!this.#disposed && this.#mediaId === mediaId) {
        this.#batches = appendBufferedBatch(this.#batches, loaded);
      }
    } finally {
      this.#loading.delete(requestKey);
    }
  }

  public draw(index: number): void {
    const frame = this.frame(index);
    if (frame === undefined) {
      throw new OverlayPlaybackError(`Frame ${index + 1} is not available yet.`);
    }
    const context = this.#canvas.getContext("2d", { alpha: true });
    if (context === null) throw new OverlayPlaybackError("The overlay canvas is unavailable.");
    context.clearRect(0, 0, this.#canvas.width, this.#canvas.height);
    context.drawImage(frame.image, 0, 0, this.#canvas.width, this.#canvas.height);
  }

  public clear(): void {
    this.#disposed = true;
    this.#loading.clear();
    this.#batches = [];
  }

  #selectMedia(mediaId: MediaId): void {
    if (this.#mediaId === mediaId) return;
    this.#mediaId = mediaId;
    this.#totalFrames = 0;
    this.#loading.clear();
    this.#batches = [];
  }

  async #loadBatch(mediaId: MediaId, offset: number): Promise<BufferedFrameBatch<LoadedFrame>> {
    const batch = await getFrameBatch(mediaId, offset, FRAME_BATCH_SIZE);
    if (batch.frames.length === 0) throw new OverlayPlaybackError("The frame batch is empty.");
    if (!this.#disposed && this.#mediaId === mediaId) {
      this.#totalFrames = batch.totalFrames;
      this.#canvas.width = batch.width;
      this.#canvas.height = batch.height;
    }
    const frames = await Promise.all(batch.frames.map(loadFrame));
    return { offset, frames };
  }
}

function loadFrame(frame: FrameAsset): Promise<LoadedFrame> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.decoding = "async";
    image.onload = () => resolve({ ...frame, image });
    image.onerror = () =>
      reject(new OverlayPlaybackError(`Frame ${frame.index + 1} could not be loaded.`));
    image.src = frame.path;
  });
}
