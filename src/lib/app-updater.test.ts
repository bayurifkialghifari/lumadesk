import type { DownloadEvent } from "@tauri-apps/plugin-updater";
import { get } from "svelte/store";
import { describe, expect, it, vi } from "vitest";
import {
  type AppUpdaterState,
  createAppUpdater,
  type UpdateCandidate,
  type UpdaterGateway,
} from "./app-updater";

function updateCandidate(
  downloadAndInstall: (onEvent?: (event: DownloadEvent) => void) => Promise<void>,
): UpdateCandidate {
  return {
    version: "0.6.0",
    downloadAndInstall,
    close: vi.fn(async () => undefined),
  };
}

describe("app updater", () => {
  it("exposes a discovered native update", async () => {
    const candidate = updateCandidate(async () => undefined);
    const gateway: UpdaterGateway = {
      check: vi.fn(async () => candidate),
      relaunch: vi.fn(async () => undefined),
    };
    const updater = createAppUpdater({
      nativeAvailable: true,
      currentVersion: "0.5.1",
      gateway,
    });

    await updater.check();

    expect(get(updater)).toEqual({
      status: "available",
      currentVersion: "0.5.1",
      nextVersion: "0.6.0",
    });
    updater.destroy();
    expect(candidate.close).toHaveBeenCalledOnce();
  });

  it("reports download progress before installing and relaunching", async () => {
    let finishDownload: (() => void) | undefined;
    const downloadGate = new Promise<void>((resolve) => {
      finishDownload = resolve;
    });
    const candidate = updateCandidate(async (onEvent) => {
      onEvent?.({ event: "Started", data: { contentLength: 200 } });
      onEvent?.({ event: "Progress", data: { chunkLength: 80 } });
      await downloadGate;
      onEvent?.({ event: "Finished" });
    });
    const gateway: UpdaterGateway = {
      check: vi.fn(async () => candidate),
      relaunch: vi.fn(async () => undefined),
    };
    const updater = createAppUpdater({
      nativeAvailable: true,
      currentVersion: "0.5.1",
      gateway,
    });
    await updater.check();

    const installing = updater.install();
    await Promise.resolve();

    expect(get(updater)).toEqual({
      status: "downloading",
      currentVersion: "0.5.1",
      nextVersion: "0.6.0",
      progressPercent: 40,
    });
    finishDownload?.();
    await installing;
    expect(get(updater).status).toBe("installing");
    expect(gateway.relaunch).toHaveBeenCalledOnce();
    updater.destroy();
    expect(candidate.close).not.toHaveBeenCalled();
  });

  it("treats a zero-byte total as unknown download progress", async () => {
    const candidate = updateCandidate(async (onEvent) => {
      onEvent?.({ event: "Started", data: { contentLength: 0 } });
      onEvent?.({ event: "Progress", data: { chunkLength: 20 } });
      throw new Error("stop after observing progress");
    });
    const updater = createAppUpdater({
      nativeAvailable: true,
      currentVersion: "0.5.1",
      gateway: {
        check: vi.fn(async () => candidate),
        relaunch: vi.fn(async () => undefined),
      },
    });
    await updater.check();

    const states: AppUpdaterState[] = [];
    const unsubscribe = updater.subscribe((value) => states.push(value));
    await updater.install();

    expect(
      states.some((state) => state.status === "downloading" && state.progressPercent === null),
    ).toBe(true);
    unsubscribe();
  });

  it("keeps the update available when installation fails", async () => {
    const failure = new Error("download failed");
    const onInstallError = vi.fn();
    const candidate = updateCandidate(async () => {
      throw failure;
    });
    const updater = createAppUpdater({
      nativeAvailable: true,
      currentVersion: "0.5.1",
      gateway: {
        check: vi.fn(async () => candidate),
        relaunch: vi.fn(async () => undefined),
      },
      onInstallError,
    });
    await updater.check();

    await updater.install();

    expect(get(updater).status).toBe("available");
    expect(onInstallError).toHaveBeenCalledWith(failure);
  });

  it("offers a restart retry when relaunching fails after installation", async () => {
    const failure = new Error("relaunch failed");
    const onInstallError = vi.fn();
    const onRestartError = vi.fn();
    const candidate = updateCandidate(async () => undefined);
    const relaunch = vi
      .fn<() => Promise<void>>()
      .mockRejectedValueOnce(failure)
      .mockResolvedValueOnce(undefined);
    const updater = createAppUpdater({
      nativeAvailable: true,
      currentVersion: "0.5.1",
      gateway: {
        check: vi.fn(async () => candidate),
        relaunch,
      },
      onInstallError,
      onRestartError,
    });
    await updater.check();

    await updater.install();

    expect(get(updater)).toEqual({
      status: "restart-required",
      currentVersion: "0.5.1",
      nextVersion: "0.6.0",
    });
    expect(onInstallError).not.toHaveBeenCalled();
    expect(onRestartError).toHaveBeenCalledWith(failure);
    expect(candidate.close).not.toHaveBeenCalled();

    await updater.restart();

    expect(relaunch).toHaveBeenCalledTimes(2);
    expect(get(updater).status).toBe("installing");
  });
});
