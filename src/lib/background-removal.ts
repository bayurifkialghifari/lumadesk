const COLOR_BUCKET_SHIFT = 5;
const COLOR_BUCKETS_PER_CHANNEL = 8;
const COLOR_BUCKET_COUNT = 512;
const TRANSPARENT_DISTANCE = 28;
const BACKGROUND_DISTANCE = 72;

interface RgbColor {
  readonly red: number;
  readonly green: number;
  readonly blue: number;
}

export class BackgroundRemovalError extends Error {
  public constructor(message: string) {
    super(message);
    this.name = "BackgroundRemovalError";
  }
}

export function removeEdgeConnectedBackground(
  pixels: Uint8ClampedArray,
  width: number,
  height: number,
): void {
  const pixelCount = width * height;
  if (!Number.isInteger(width) || !Number.isInteger(height) || width <= 0 || height <= 0) {
    throw new BackgroundRemovalError("Background removal requires valid frame dimensions.");
  }
  if (pixels.length !== pixelCount * 4) {
    throw new BackgroundRemovalError("Background removal received an invalid pixel buffer.");
  }

  const estimatedBackground = estimateBackground(pixels, width, height);
  if (estimatedBackground === null) return;
  const background: RgbColor = estimatedBackground;

  const visited = new Uint8Array(pixelCount);
  const queue = new Uint32Array(pixelCount);
  let head = 0;
  let tail = 0;

  function enqueue(pixelIndex: number): void {
    if (visited[pixelIndex] !== 0) return;
    visited[pixelIndex] = 1;
    const offset = pixelIndex * 4;
    const alpha = pixels[offset + 3] ?? 0;
    if (alpha === 0) {
      queue[tail] = pixelIndex;
      tail += 1;
      return;
    }

    const distance = colorDistance(
      pixels[offset] ?? 0,
      pixels[offset + 1] ?? 0,
      pixels[offset + 2] ?? 0,
      background,
    );
    if (distance > BACKGROUND_DISTANCE) return;

    const feather = Math.max(
      0,
      Math.min(1, (distance - TRANSPARENT_DISTANCE) / (BACKGROUND_DISTANCE - TRANSPARENT_DISTANCE)),
    );
    pixels[offset + 3] = Math.round(alpha * feather);
    queue[tail] = pixelIndex;
    tail += 1;
  }

  for (let x = 0; x < width; x += 1) {
    enqueue(x);
    if (height > 1) enqueue((height - 1) * width + x);
  }
  for (let y = 1; y < height - 1; y += 1) {
    enqueue(y * width);
    if (width > 1) enqueue(y * width + width - 1);
  }

  while (head < tail) {
    const pixelIndex = queue[head] ?? 0;
    head += 1;
    const x = pixelIndex % width;
    const y = Math.floor(pixelIndex / width);
    if (x > 0) enqueue(pixelIndex - 1);
    if (x + 1 < width) enqueue(pixelIndex + 1);
    if (y > 0) enqueue(pixelIndex - width);
    if (y + 1 < height) enqueue(pixelIndex + width);
  }
}

function estimateBackground(
  pixels: Uint8ClampedArray,
  width: number,
  height: number,
): RgbColor | null {
  const counts = new Uint32Array(COLOR_BUCKET_COUNT);
  const redSums = new Uint32Array(COLOR_BUCKET_COUNT);
  const greenSums = new Uint32Array(COLOR_BUCKET_COUNT);
  const blueSums = new Uint32Array(COLOR_BUCKET_COUNT);

  function sample(pixelIndex: number): void {
    const offset = pixelIndex * 4;
    if ((pixels[offset + 3] ?? 0) === 0) return;
    const red = pixels[offset] ?? 0;
    const green = pixels[offset + 1] ?? 0;
    const blue = pixels[offset + 2] ?? 0;
    const bucket =
      (red >> COLOR_BUCKET_SHIFT) * COLOR_BUCKETS_PER_CHANNEL * COLOR_BUCKETS_PER_CHANNEL +
      (green >> COLOR_BUCKET_SHIFT) * COLOR_BUCKETS_PER_CHANNEL +
      (blue >> COLOR_BUCKET_SHIFT);
    counts[bucket] = (counts[bucket] ?? 0) + 1;
    redSums[bucket] = (redSums[bucket] ?? 0) + red;
    greenSums[bucket] = (greenSums[bucket] ?? 0) + green;
    blueSums[bucket] = (blueSums[bucket] ?? 0) + blue;
  }

  for (let x = 0; x < width; x += 1) {
    sample(x);
    if (height > 1) sample((height - 1) * width + x);
  }
  for (let y = 1; y < height - 1; y += 1) {
    sample(y * width);
    if (width > 1) sample(y * width + width - 1);
  }

  let dominantBucket = 0;
  let dominantCount = 0;
  for (let bucket = 0; bucket < COLOR_BUCKET_COUNT; bucket += 1) {
    const count = counts[bucket] ?? 0;
    if (count > dominantCount) {
      dominantBucket = bucket;
      dominantCount = count;
    }
  }
  if (dominantCount === 0) return null;

  return {
    red: Math.round((redSums[dominantBucket] ?? 0) / dominantCount),
    green: Math.round((greenSums[dominantBucket] ?? 0) / dominantCount),
    blue: Math.round((blueSums[dominantBucket] ?? 0) / dominantCount),
  };
}

function colorDistance(red: number, green: number, blue: number, background: RgbColor): number {
  return Math.hypot(red - background.red, green - background.green, blue - background.blue);
}
