import { applyAlphaMask } from "./alpha-mask";
import { getAiMaskBatch, getFrameBatch } from "./api";
import { removeEdgeConnectedBackground } from "./background-removal";
import type { AiModel, EditRecipe, FrameAsset, MediaId } from "./contracts";
import { editFilter } from "./edit-recipes";
import {
  appendBufferedBatch,
  type BufferedFrameBatch,
  FRAME_BATCH_SIZE,
  findBufferedFrame,
} from "./overlay-player";

interface LoadedFrame extends FrameAsset {
  readonly image: HTMLImageElement | ImageBitmap;
  backgroundRemovedImage?: HTMLCanvasElement;
  aiMasks?: Partial<Record<AiModel, HTMLImageElement | ImageBitmap>>;
  aiRemovedImages?: Partial<Record<AiModel, HTMLCanvasElement>>;
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
  readonly #maskLoading = new Map<string, Promise<void>>();
  #batches: readonly BufferedFrameBatch<LoadedFrame>[] = [];
  #mediaId: MediaId | null = null;
  #totalFrames = 0;
  #disposed = false;
  #revision = 0;

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
    const revision = this.#revision;
    if (this.#batches.some((batch) => batch.offset === offset)) return;
    const requestKey = `${revision}:${mediaId}:${offset}`;
    const inFlight = this.#loading.get(requestKey);
    if (inFlight !== undefined) {
      await inFlight;
      return;
    }

    const request = this.#loadBatch(mediaId, offset, revision);
    this.#loading.set(requestKey, request);
    try {
      const loaded = await request;
      if (this.#isCurrent(mediaId, revision)) {
        this.#batches = appendBufferedBatch(this.#batches, loaded);
      } else {
        closeLoadedFrames(loaded.frames);
      }
    } finally {
      if (this.#loading.get(requestKey) === request) this.#loading.delete(requestKey);
    }
  }

  public draw(index: number, recipe: EditRecipe, removeBackground: boolean): void {
    const frame = this.frame(index);
    if (frame === undefined) {
      throw new OverlayPlaybackError(`Frame ${index + 1} is not available yet.`);
    }
    const context = this.#canvas.getContext("2d", { alpha: true });
    if (context === null) throw new OverlayPlaybackError("The overlay canvas is unavailable.");
    const model = aiModelFor(recipe);
    const image =
      model !== null
        ? (aiBackgroundRemovedImage(frame, model) ?? backgroundRemovedImage(frame))
        : removeBackground || recipe.backgroundRemoval === "edge"
          ? backgroundRemovedImage(frame)
          : frame.image;
    const dimensions = imageDimensions(image);
    const crop = normalizedCrop(recipe, dimensions.width, dimensions.height);
    const rotated = recipe.rotationDegrees === 90 || recipe.rotationDegrees === 270;
    this.#canvas.width = Math.max(1, Math.round(rotated ? crop.height : crop.width));
    this.#canvas.height = Math.max(1, Math.round(rotated ? crop.width : crop.height));
    context.clearRect(0, 0, this.#canvas.width, this.#canvas.height);
    context.save();
    context.filter = editFilter(recipe);
    context.translate(this.#canvas.width / 2, this.#canvas.height / 2);
    context.rotate((recipe.rotationDegrees * Math.PI) / 180);
    context.scale(recipe.flipHorizontal ? -1 : 1, 1);
    context.drawImage(
      image,
      crop.x,
      crop.y,
      crop.width,
      crop.height,
      -crop.width / 2,
      -crop.height / 2,
      crop.width,
      crop.height,
    );
    context.restore();
  }

  public async ensureMasks(mediaId: MediaId, model: AiModel, offset: number): Promise<void> {
    this.#selectMedia(mediaId);
    const revision = this.#revision;
    await this.ensure(mediaId, offset);
    if (!this.#isCurrent(mediaId, revision)) return;
    const requestKey = `${revision}:${mediaId}:${model}:${offset}`;
    const inFlight = this.#maskLoading.get(requestKey);
    if (inFlight !== undefined) {
      await inFlight;
      return;
    }
    const request = this.#loadMasks(mediaId, model, offset, revision);
    this.#maskLoading.set(requestKey, request);
    try {
      await request;
    } finally {
      if (this.#maskLoading.get(requestKey) === request) this.#maskLoading.delete(requestKey);
    }
  }

  public invalidate(mediaId?: MediaId): void {
    if (mediaId !== undefined && this.#mediaId !== mediaId) return;
    this.#revision += 1;
    this.#totalFrames = 0;
    this.#loading.clear();
    this.#maskLoading.clear();
    closeBufferedBatches(this.#batches);
    this.#batches = [];
  }

  public clear(): void {
    this.#disposed = true;
    this.#revision += 1;
    this.#mediaId = null;
    this.#loading.clear();
    this.#maskLoading.clear();
    closeBufferedBatches(this.#batches);
    this.#batches = [];
  }

  #selectMedia(mediaId: MediaId): void {
    if (this.#mediaId === mediaId) return;
    this.#revision += 1;
    this.#mediaId = mediaId;
    this.#totalFrames = 0;
    this.#loading.clear();
    this.#maskLoading.clear();
    closeBufferedBatches(this.#batches);
    this.#batches = [];
  }

  #isCurrent(mediaId: MediaId, revision: number): boolean {
    return !this.#disposed && this.#mediaId === mediaId && this.#revision === revision;
  }

  async #loadBatch(
    mediaId: MediaId,
    offset: number,
    revision: number,
  ): Promise<BufferedFrameBatch<LoadedFrame>> {
    const batch = await getFrameBatch(mediaId, offset, FRAME_BATCH_SIZE);
    if (batch.frames.length === 0) throw new OverlayPlaybackError("The frame batch is empty.");
    if (!this.#isCurrent(mediaId, revision)) return { offset, frames: [] };
    this.#totalFrames = batch.totalFrames;
    this.#canvas.width = batch.width;
    this.#canvas.height = batch.height;
    const frames = await Promise.all(batch.frames.map(loadFrame));
    return { offset, frames };
  }

  async #loadMasks(
    mediaId: MediaId,
    model: AiModel,
    offset: number,
    revision: number,
  ): Promise<void> {
    const batch = await getAiMaskBatch(mediaId, model, offset, FRAME_BATCH_SIZE);
    if (!this.#isCurrent(mediaId, revision)) return;
    const masks = await Promise.all(
      batch.masks.map(async (mask) => ({
        ...mask,
        image: await loadOriginCleanImage(mask.path, mask.index),
      })),
    );
    if (!this.#isCurrent(mediaId, revision)) {
      closeImages(masks.map((mask) => mask.image));
      return;
    }
    for (const mask of masks) {
      const frame = this.frame(mask.index);
      if (frame === undefined) continue;
      frame.aiMasks = { ...frame.aiMasks, [model]: mask.image };
      if (frame.aiRemovedImages !== undefined) delete frame.aiRemovedImages[model];
    }
  }
}

function closeLoadedFrames(frames: readonly LoadedFrame[]): void {
  for (const frame of frames) {
    closeImages([frame.image, ...Object.values(frame.aiMasks ?? {}).filter(isLoadedImage)]);
  }
}

function closeBufferedBatches(batches: readonly BufferedFrameBatch<LoadedFrame>[]): void {
  for (const batch of batches) closeLoadedFrames(batch.frames);
}

function isLoadedImage(
  image: HTMLImageElement | ImageBitmap | undefined,
): image is HTMLImageElement | ImageBitmap {
  return image !== undefined;
}

function closeImages(images: readonly (HTMLImageElement | ImageBitmap)[]): void {
  for (const image of images) {
    if ("close" in image && typeof image.close === "function") image.close();
  }
}

function backgroundRemovedImage(frame: LoadedFrame): HTMLCanvasElement {
  if (frame.backgroundRemovedImage !== undefined) return frame.backgroundRemovedImage;
  const canvas = document.createElement("canvas");
  const dimensions = imageDimensions(frame.image);
  canvas.width = dimensions.width;
  canvas.height = dimensions.height;
  const context = canvas.getContext("2d", { alpha: true });
  if (context === null) throw new OverlayPlaybackError("Background removal is unavailable.");
  context.drawImage(frame.image, 0, 0);
  const imageData = context.getImageData(0, 0, canvas.width, canvas.height);
  removeEdgeConnectedBackground(imageData.data, canvas.width, canvas.height);
  context.putImageData(imageData, 0, 0);
  frame.backgroundRemovedImage = canvas;
  return canvas;
}

function aiBackgroundRemovedImage(
  frame: LoadedFrame,
  model: AiModel,
): HTMLCanvasElement | undefined {
  const cached = frame.aiRemovedImages?.[model];
  if (cached !== undefined) return cached;
  const mask = frame.aiMasks?.[model];
  if (mask === undefined) return undefined;
  const canvas = document.createElement("canvas");
  const dimensions = imageDimensions(frame.image);
  canvas.width = dimensions.width;
  canvas.height = dimensions.height;
  const context = canvas.getContext("2d", { alpha: true });
  if (context === null) throw new OverlayPlaybackError("Background removal is unavailable.");
  context.drawImage(frame.image, 0, 0, canvas.width, canvas.height);
  const source = context.getImageData(0, 0, canvas.width, canvas.height);
  const maskCanvas = document.createElement("canvas");
  maskCanvas.width = canvas.width;
  maskCanvas.height = canvas.height;
  const maskContext = maskCanvas.getContext("2d", { alpha: false });
  if (maskContext === null) throw new OverlayPlaybackError("The AI mask is unavailable.");
  maskContext.drawImage(mask, 0, 0, canvas.width, canvas.height);
  const maskPixels = maskContext.getImageData(0, 0, canvas.width, canvas.height).data;
  applyAlphaMask(source.data, maskPixels);
  context.putImageData(source, 0, 0);
  frame.aiRemovedImages = { ...frame.aiRemovedImages, [model]: canvas };
  return canvas;
}

export function loadFrame(frame: FrameAsset): Promise<LoadedFrame> {
  return loadOriginCleanFrame(frame).catch(() => loadImageElement(frame));
}

async function loadOriginCleanFrame(frame: FrameAsset): Promise<LoadedFrame> {
  const image = await loadOriginCleanImage(frame.path, frame.index);
  return { ...frame, image };
}

async function loadOriginCleanImage(
  path: string,
  index: number,
): Promise<HTMLImageElement | ImageBitmap> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new OverlayPlaybackError(`Frame ${index + 1} could not be loaded.`);
  }
  return createImageBitmap(await response.blob());
}

function loadImageElement(frame: FrameAsset): Promise<LoadedFrame> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.decoding = "async";
    image.crossOrigin = "anonymous";
    image.onload = () => resolve({ ...frame, image });
    image.onerror = () =>
      reject(new OverlayPlaybackError(`Frame ${frame.index + 1} could not be loaded.`));
    image.src = frame.path;
  });
}

function imageDimensions(image: HTMLImageElement | ImageBitmap | HTMLCanvasElement): {
  readonly width: number;
  readonly height: number;
} {
  if (image instanceof HTMLImageElement) {
    return { width: image.naturalWidth, height: image.naturalHeight };
  }
  return { width: image.width, height: image.height };
}

function normalizedCrop(
  recipe: EditRecipe,
  sourceWidth: number,
  sourceHeight: number,
): { readonly x: number; readonly y: number; readonly width: number; readonly height: number } {
  if (recipe.crop === null) {
    return { x: 0, y: 0, width: sourceWidth, height: sourceHeight };
  }
  const x = Math.min(sourceWidth - 1, Math.max(0, recipe.crop.x));
  const y = Math.min(sourceHeight - 1, Math.max(0, recipe.crop.y));
  return {
    x,
    y,
    width: Math.max(1, Math.min(recipe.crop.width, sourceWidth - x)),
    height: Math.max(1, Math.min(recipe.crop.height, sourceHeight - y)),
  };
}

function aiModelFor(recipe: EditRecipe): AiModel | null {
  if (recipe.backgroundRemoval === "ai_fast") return "fast";
  if (recipe.backgroundRemoval === "ai_quality") return "quality";
  return null;
}
