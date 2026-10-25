import { NativeCommandError } from "./api";
import type { MediaItem, MediaKind } from "./contracts";

const MEDIA_KIND_LABEL: Readonly<Record<MediaKind, string>> = {
  gif: "GIF",
  apng: "APNG",
  webp: "WebP",
  png: "PNG",
  jpeg: "JPG",
};

const decimal = new Intl.NumberFormat("en-US", { maximumFractionDigits: 1 });
const date = new Intl.DateTimeFormat("en-US", {
  day: "numeric",
  month: "short",
  year: "numeric",
});

export function mediaKindLabel(kind: MediaKind): string {
  return MEDIA_KIND_LABEL[kind];
}

export function mediaDimensions(item: MediaItem): string {
  return `${item.width} × ${item.height} px`;
}

export function mediaDuration(item: MediaItem): string {
  if (item.frameCount === 1 || item.durationMs === 0) {
    return "Static image";
  }
  return `${decimal.format(item.durationMs / 1000)} seconds per loop`;
}

export function mediaFrameLabel(frameCount: number): string {
  return frameCount === 1 ? "1 frame" : `${frameCount} frame`;
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
