import { NativeCommandError } from "./api";
import type { MediaFormat, MediaItem } from "./contracts";

const MEDIA_FORMAT_LABEL: Readonly<Record<MediaFormat, string>> = {
  gif: "GIF",
  apng: "APNG",
  webp: "WebP",
  png: "PNG",
  jpeg: "JPG",
  mp4: "MP4",
  webm: "WebM",
  mp3: "MP3",
  wav: "WAV",
  ogg: "OGG",
};

const decimal = new Intl.NumberFormat("en-US", { maximumFractionDigits: 1 });
const date = new Intl.DateTimeFormat("en-US", {
  day: "numeric",
  month: "short",
  year: "numeric",
});

export function mediaFormatLabel(format: MediaFormat): string {
  return MEDIA_FORMAT_LABEL[format];
}

export function mediaDimensions(item: MediaItem): string {
  return `${item.width} × ${item.height} px`;
}

export function mediaDuration(item: MediaItem): string {
  if (item.assetKind === "audio" || item.assetKind === "video") {
    return `${decimal.format(item.durationMs / 1000)} seconds`;
  }
  if (item.frameCount === 1 || item.durationMs === 0) {
    return "Static image";
  }
  return `${decimal.format(item.durationMs / 1000)} seconds per loop`;
}

export function mediaFrameLabel(frameCount: number): string {
  return frameCount === 1 ? "1 frame" : `${frameCount} frames`;
}

export function mediaDate(value: string): string {
  return date.format(new Date(value));
}

export function userErrorMessage(error: unknown, fallback: string): string {
  if (error instanceof NativeCommandError) {
    return error.message;
  }
  if (error instanceof Error && error.message.trim() !== "") {
    return error.message;
  }
  return fallback;
}

export function filenameFromPath(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function parseTags(value: string): string[] {
  const tags = value
    .split(",")
    .map((tag) => tag.trim())
    .filter((tag) => tag.length > 0);
  return [...new Set(tags)];
}
