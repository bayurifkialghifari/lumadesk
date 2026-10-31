import type { DownloadEvent } from "@tauri-apps/plugin-updater";
import { get } from "svelte/store";
import { describe, expect, it, vi } from "vitest";
import {
  type AppUpdaterState,
  createAppUpdater,
  type UpdateCandidate,
  type UpdaterGateway,
} from "./app-updater";

interface Deferred<T> {
  readonly promise: Promise<T>;
  resolve(value: T): void;
  reject(error: unknown): void;
}

function deferred<T>(): Deferred<T> {
  let resolvePromise: (value: T) => void = () => undefined;
  let rejectPromise: (error: unknown) => void = () => undefined;
  const promise = new Promise<T>((resolve, reject) => {
    resolvePromise = resolve;
    rejectPromise = reject;
  });
  return { promise, resolve: resolvePromise, reject: rejectPromise };
}

function updateCandidate(
  downloadAndInstall: (onEvent?: (event: DownloadEvent) => void) => Promise<void>,
  version = "0.6.2",
): UpdateCandidate {
  return {
    version,
    downloadAndInstall: vi.fn(downloadAndInstall),
    close: vi.fn(async () => undefined),
  };
}

function updaterWith(
  gateway: UpdaterGateway,
  callbacks: {
    onInstallError?: (error: unknown) => void;
    onRetryError?: (error: unknown) => void;
    onRestartError?: (error: unknown) => void;
  } = {},
) {
  return createAppUpdater({
    nativeAvailable: true,
    currentVersion: "0.6.1",
    gateway,
    ...callbacks,
  });
}

describe("app updater", () => {
  it("exposes a discovered native update and closes it on destroy", async () => {
    const candidate = updateCandidate(async () => undefined);
    const updater = updaterWith({
      check: vi.fn(async () => candidate),
      relaunch: vi.fn(async () => undefined),
    });

    await updater.check();

    expect(get(updater)).toEqual({
      status: "available",
      currentVersion: "0.6.1",
      nextVersion: "0.6.2",
    });
    updater.destroy();
    await vi.waitFor(() => expect(candidate.close).toHaveBeenCalledOnce());
  });

  it("reports download progress before installing and relaunching", async () => {
    const downloadGate = deferred<void>();
    const candidate = updateCandidate(async (onEvent) => {
      onEvent?.({ event: "Started", data: { contentLength: 200 } });
      onEvent?.({ event: "Progress", data: { chunkLength: 80 } });
      await downloadGate.promise;
      onEvent?.({ event: "Finished" });
    });
    const gateway: UpdaterGateway = {
      check: vi.fn(async () => candidate),
      relaunch: vi.fn(async () => undefined),
    };
    const updater = updaterWith(gateway);
    await updater.check();

    const installing = updater.install();
    await vi.waitFor(() => expect(get(updater).status).toBe("downloading"));

    expect(get(updater)).toEqual({
      status: "downloading",
      currentVersion: "0.6.1",
      nextVersion: "0.6.2",
      progressPercent: 40,
    });
    downloadGate.resolve(undefined);
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
    const updater = updaterWith({
      check: vi.fn(async () => candidate),
      relaunch: vi.fn(async () => undefined),
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

  it("discards the failed candidate and retries with a fresh check", async () => {
    const failure = new Error("download failed");
    const stale = updateCandidate(async () => {
      throw failure;
    });
    const fresh = updateCandidate(async () => undefined);
    const check = vi.fn<() => Promise<UpdateCandidate | null>>();
    check.mockResolvedValueOnce(stale).mockResolvedValueOnce(fresh);
    const onInstallError = vi.fn();
    const gateway = { check, relaunch: vi.fn(async () => undefined) };
    const updater = updaterWith(gateway, { onInstallError });
    await updater.check();

    await updater.install();

    expect(get(updater)).toEqual({
      status: "failed",
      currentVersion: "0.6.1",
      nextVersion: "0.6.2",
    });
    expect(stale.close).toHaveBeenCalledOnce();
    expect(onInstallError).toHaveBeenCalledWith(failure);

    await updater.retry();

    expect(check).toHaveBeenCalledTimes(2);
    expect(fresh.downloadAndInstall).toHaveBeenCalledOnce();
    expect(gateway.relaunch).toHaveBeenCalledOnce();
    expect(stale.close).toHaveBeenCalledOnce();
    expect(fresh.close).not.toHaveBeenCalled();
  });

  it("keeps recovery available when a retry check fails", async () => {
    const stale = updateCandidate(async () => {
      throw new Error("install failed");
    });
    const retryFailure = new Error("The update service is unavailable.");
    const check = vi
      .fn<() => Promise<UpdateCandidate | null>>()
      .mockResolvedValueOnce(stale)
      .mockRejectedValueOnce(retryFailure);
    const onRetryError = vi.fn();
    const updater = updaterWith(
      { check, relaunch: vi.fn(async () => undefined) },
      { onRetryError },
    );
    await updater.check();
    await updater.install();

    await updater.retry();

    expect(get(updater).status).toBe("failed");
    expect(onRetryError).toHaveBeenCalledWith(retryFailure);
  });

  it("keeps recovery available when no compatible retry package exists", async () => {
    const stale = updateCandidate(async () => {
      throw new Error("install failed");
    });
    const check = vi
      .fn<() => Promise<UpdateCandidate | null>>()
      .mockResolvedValueOnce(stale)
      .mockResolvedValueOnce(null);
    const onRetryError = vi.fn();
    const updater = updaterWith(
      { check, relaunch: vi.fn(async () => undefined) },
      { onRetryError },
    );
    await updater.check();
    await updater.install();

    await updater.retry();

    expect(get(updater)).toMatchObject({ status: "failed", nextVersion: "0.6.2" });
    expect(onRetryError).toHaveBeenCalledWith(
      expect.objectContaining({ message: expect.stringContaining("No compatible update package") }),
    );
  });

  it("closes a candidate returned after destruction without mutating state", async () => {
    const checkGate = deferred<UpdateCandidate | null>();
    const candidate = updateCandidate(async () => undefined);
    const updater = updaterWith({
      check: vi.fn(() => checkGate.promise),
      relaunch: vi.fn(async () => undefined),
    });
    const states: AppUpdaterState[] = [];
    updater.subscribe((value) => states.push(value));

    const checking = updater.check();
    updater.destroy();
    checkGate.resolve(candidate);
    await checking;

    expect(candidate.close).toHaveBeenCalledOnce();
    expect(states.at(-1)?.status).toBe("checking");
  });

  it("ignores download events and errors after destruction during install", async () => {
    const downloadGate = deferred<void>();
    let sendEvent: ((event: DownloadEvent) => void) | undefined;
    const candidate = updateCandidate(async (onEvent) => {
      sendEvent = onEvent;
      await downloadGate.promise;
      throw new Error("late failure");
    });
    const onInstallError = vi.fn();
    const updater = updaterWith(
      {
        check: vi.fn(async () => candidate),
        relaunch: vi.fn(async () => undefined),
      },
      { onInstallError },
    );
    await updater.check();
    const states: AppUpdaterState[] = [];
    updater.subscribe((value) => states.push(value));

    const installing = updater.install();
    await vi.waitFor(() => expect(candidate.downloadAndInstall).toHaveBeenCalledOnce());
    updater.destroy();
    sendEvent?.({ event: "Progress", data: { chunkLength: 20 } });
    downloadGate.resolve(undefined);
    await installing;

    expect(candidate.close).toHaveBeenCalledOnce();
    expect(onInstallError).not.toHaveBeenCalled();
    expect(states.at(-1)).toMatchObject({ status: "downloading", progressPercent: null });
  });

  it("ignores duplicate install and retry clicks while an operation is active", async () => {
    const downloadGate = deferred<void>();
    const candidate = updateCandidate(async () => downloadGate.promise);
    const check = vi.fn(async () => candidate);
    const updater = updaterWith({ check, relaunch: vi.fn(async () => undefined) });
    await updater.check();

    const first = updater.install();
    await Promise.all([updater.install(), updater.retry()]);

    expect(candidate.downloadAndInstall).toHaveBeenCalledOnce();
    expect(check).toHaveBeenCalledOnce();
    downloadGate.resolve(undefined);
    await first;
  });

  it("preserves restart-required while restart attempts fail", async () => {
    const candidate = updateCandidate(async () => undefined);
    const firstFailure = new Error("relaunch failed");
    const secondFailure = new Error("relaunch still failed");
    const relaunch = vi
      .fn<() => Promise<void>>()
      .mockRejectedValueOnce(firstFailure)
      .mockRejectedValueOnce(secondFailure);
    const onRestartError = vi.fn();
    const updater = updaterWith(
      { check: vi.fn(async () => candidate), relaunch },
      { onRestartError },
    );
    await updater.check();

    await updater.install();
    await updater.restart();

    expect(get(updater)).toEqual({
      status: "restart-required",
      currentVersion: "0.6.1",
      nextVersion: "0.6.2",
    });
    expect(onRestartError).toHaveBeenNthCalledWith(1, firstFailure);
    expect(onRestartError).toHaveBeenNthCalledWith(2, secondFailure);
    expect(candidate.close).not.toHaveBeenCalled();
  });

  it("does not invoke restart callbacks after destruction", async () => {
    const relaunchGate = deferred<void>();
    const candidate = updateCandidate(async () => undefined);
    const onRestartError = vi.fn();
    const updater = updaterWith(
      { check: vi.fn(async () => candidate), relaunch: vi.fn(() => relaunchGate.promise) },
      { onRestartError },
    );
    await updater.check();

    const installing = updater.install();
    await vi.waitFor(() => expect(get(updater).status).toBe("installing"));
    updater.destroy();
    relaunchGate.reject(new Error("late restart failure"));
    await installing;

    expect(onRestartError).not.toHaveBeenCalled();
  });
});
