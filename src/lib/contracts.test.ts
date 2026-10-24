import { describe, expect, it } from "vitest";
import { desktopOverlaySchema, mediaItemSchema, settingsSchema } from "./contracts";

const media = {
  id: "2db87e5a-5968-4c30-89e9-bf1980e8bb5f",
  contentHash: "a".repeat(64),
  displayName: "Lampu Biru",
  managedPath: "2db87e5a-5968-4c30-89e9-bf1980e8bb5f.png",
  mediaKind: "png",
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
      launchAtLogin: false,
      restoreOverlays: true,
      closeToTray: true,
      collectionView: "grid",
      defaultOverlayWidth: 240,
      defaultOpacity: 1,
      defaultSpeed: 5,
      cacheLimitMb: 512,
    };
    expect(() => settingsSchema.parse(settings)).toThrow();
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
});
