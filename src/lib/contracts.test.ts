import { describe, expect, it } from "vitest";
import {
  aiMaskBatchSchema,
  aiModelStatusSchema,
  desktopOverlaySchema,
  desktopSceneSchema,
  mediaItemSchema,
  settingsSchema,
} from "./contracts";

const media = {
  id: "2db87e5a-5968-4c30-89e9-bf1980e8bb5f",
  contentHash: "a".repeat(64),
  displayName: "Blue Light",
  managedPath: "2db87e5a-5968-4c30-89e9-bf1980e8bb5f.png",
  assetKind: "still",
  mediaFormat: "png",
  width: 320,
  height: 240,
  frameCount: 1,
  durationMs: 0,
  isFavorite: false,
  tags: ["pixel"],
  createdAt: "2026-10-24T08:00:00Z",
  updatedAt: "2026-10-24T08:00:00Z",
};

describe("native command contracts", () => {
  it("accepts a complete media item", () => {
    expect(mediaItemSchema.parse(media)).toEqual(media);
  });

  it("rejects impossible media metadata", () => {
    expect(() => mediaItemSchema.parse({ ...media, frameCount: 0 })).toThrow();
  });

  it("rejects settings outside overlay limits", () => {
    const settings = {
      theme: "dark",
      launchAtLogin: false,
      restoreOverlays: true,
      closeToTray: true,
      collectionView: "grid",
      defaultOverlayWidth: 240,
      defaultOpacity: 1,
      defaultSpeed: 5,
    };
    expect(() => settingsSchema.parse(settings)).toThrow();
  });

  it("validates local AI model and mask payloads", () => {
    expect(
      aiModelStatusSchema.parse({
        model: "fast",
        status: "installed",
        sizeBytes: 4_574_861,
        installedBytes: 4_574_861,
        message: null,
      }),
    ).toMatchObject({ model: "fast", status: "installed" });
    expect(
      aiMaskBatchSchema.parse({
        mediaId: media.id,
        model: "quality",
        masks: [{ index: 0, path: "/data/masks/00000000.png" }],
      }).masks,
    ).toHaveLength(1);
    expect(() =>
      aiMaskBatchSchema.parse({ mediaId: media.id, model: "remote", masks: [] }),
    ).toThrow();
  });

  it("validates the combined media and overlay payload used by the desktop manager", () => {
    const overlay = {
      id: "39083219-2b78-4f49-859f-8016067e19b1",
      mediaId: media.id,
      monitorId: "Primary",
      x: 24,
      y: 40,
      width: 320,
      opacity: 0.8,
      speed: 1.25,
      flipHorizontal: false,
      removeBackground: true,
      clickThrough: false,
      isPlaying: true,
      zOrder: 0,
      isActive: true,
      createdAt: "2026-10-24T08:00:00Z",
      updatedAt: "2026-10-24T08:00:00Z",
    };

    expect(desktopOverlaySchema.parse({ overlay, media })).toEqual({ overlay, media });
    expect(() =>
      desktopOverlaySchema.parse({ overlay: { ...overlay, opacity: 1.1 }, media }),
    ).toThrow();
  });

  it("validates monitor-aware desktop scenes", () => {
    const scene = {
      id: "49ea2727-5237-4842-9d26-179074952d33",
      name: "Streaming",
      items: [
        {
          mediaId: media.id,
          monitorName: "Primary",
          monitorWidth: 1920,
          monitorHeight: 1080,
          normalizedX: 0.25,
          normalizedY: 0.5,
          width: 320,
          opacity: 0.8,
          speed: 1.25,
          flipHorizontal: false,
          removeBackground: true,
          isPlaying: true,
          clickThrough: true,
          zOrder: 0,
          isActive: false,
        },
      ],
      createdAt: "2026-10-31T08:00:00Z",
      updatedAt: "2026-10-31T08:00:00Z",
    };

    expect(desktopSceneSchema.parse(scene)).toEqual(scene);
    expect(() => desktopSceneSchema.parse({ ...scene, name: "" })).toThrow();
  });
});
