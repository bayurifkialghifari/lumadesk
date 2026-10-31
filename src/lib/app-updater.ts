import type { DownloadEvent } from "@tauri-apps/plugin-updater";
import { type Readable, writable } from "svelte/store";

export type AppUpdaterState =
  | { readonly status: "idle"; readonly currentVersion: string }
  | { readonly status: "checking"; readonly currentVersion: string }
  | {
      readonly status: "available";
      readonly currentVersion: string;
      readonly nextVersion: string;
    }
  | {
      readonly status: "downloading";
      readonly currentVersion: string;
      readonly nextVersion: string;
      readonly progressPercent: number | null;
    }
  | {
      readonly status: "installing";
      readonly currentVersion: string;
      readonly nextVersion: string;
    }
  | {
      readonly status: "failed";
      readonly currentVersion: string;
      readonly nextVersion: string;
    }
  | {
      readonly status: "restart-required";
      readonly currentVersion: string;
      readonly nextVersion: string;
    };

export interface UpdateCandidate {
  readonly version: string;
  downloadAndInstall(onEvent?: (event: DownloadEvent) => void): Promise<void>;
  close(): Promise<void>;
}

export interface UpdaterGateway {
  check(): Promise<UpdateCandidate | null>;
  relaunch(): Promise<void>;
}

export interface AppUpdaterController extends Readable<AppUpdaterState> {
  check(): Promise<void>;
  install(): Promise<void>;
  retry(): Promise<void>;
  restart(): Promise<void>;
  destroy(): void;
}

interface AppUpdaterOptions {
  readonly nativeAvailable: boolean;
  readonly currentVersion: string;
  readonly gateway?: UpdaterGateway;
  readonly onInstallError?: (error: unknown) => void;
  readonly onRetryError?: (error: unknown) => void;
  readonly onRestartError?: (error: unknown) => void;
}

const nativeGateway: UpdaterGateway = {
  async check() {
    const { check } = await import("@tauri-apps/plugin-updater");
    return check({ timeout: 10_000 });
  },
  async relaunch() {
    const { relaunch } = await import("@tauri-apps/plugin-process");
    await relaunch();
  },
};

export function createAppUpdater(options: AppUpdaterOptions): AppUpdaterController {
  const state = writable<AppUpdaterState>({
    status: "idle",
    currentVersion: options.currentVersion,
  });
  const gateway = options.gateway ?? nativeGateway;
  const closedCandidates = new WeakSet<UpdateCandidate>();
  let candidate: UpdateCandidate | null = null;
  let failedVersion: string | null = null;
  let installedVersion: string | null = null;
  let busy = false;
  let destroyed = false;
  let operation = 0;
  let downloadedBytes = 0;
  let totalBytes: number | undefined;

  function availableState(update: UpdateCandidate): AppUpdaterState {
    return {
      status: "available",
      currentVersion: options.currentVersion,
      nextVersion: update.version,
    };
  }

  function failedState(nextVersion: string): AppUpdaterState {
    return {
      status: "failed",
      currentVersion: options.currentVersion,
      nextVersion,
    };
  }

  function active(operationId: number): boolean {
    return !destroyed && operation === operationId;
  }

  function beginOperation(): number {
    busy = true;
    operation += 1;
    return operation;
  }

  function finishOperation(operationId: number): void {
    if (active(operationId)) busy = false;
  }

  async function closeCandidate(update: UpdateCandidate): Promise<void> {
    if (closedCandidates.has(update)) return;
    closedCandidates.add(update);
    try {
      await update.close();
    } catch (error) {
      console.error("LumaDesk updater cleanup failed", error);
    }
  }

  function handleDownload(
    event: DownloadEvent,
    update: UpdateCandidate,
    operationId: number,
  ): void {
    if (!active(operationId)) return;
    switch (event.event) {
      case "Started":
        downloadedBytes = 0;
        totalBytes =
          event.data.contentLength === undefined || event.data.contentLength <= 0
            ? undefined
            : event.data.contentLength;
        state.set({
          status: "downloading",
          currentVersion: options.currentVersion,
          nextVersion: update.version,
          progressPercent: totalBytes === undefined ? null : 0,
        });
        break;
      case "Progress":
        downloadedBytes += event.data.chunkLength;
        state.set({
          status: "downloading",
          currentVersion: options.currentVersion,
          nextVersion: update.version,
          progressPercent:
            totalBytes === undefined
              ? null
              : Math.min(100, Math.round((downloadedBytes / totalBytes) * 100)),
        });
        break;
      case "Finished":
        state.set({
          status: "installing",
          currentVersion: options.currentVersion,
          nextVersion: update.version,
        });
        break;
      default:
        assertNever(event);
    }
  }

  async function installCandidate(
    update: UpdateCandidate,
    operationId: number,
    onError: ((error: unknown) => void) | undefined,
  ): Promise<void> {
    state.set({
      status: "downloading",
      currentVersion: options.currentVersion,
      nextVersion: update.version,
      progressPercent: null,
    });
    let installed = false;
    try {
      await update.downloadAndInstall((event) => handleDownload(event, update, operationId));
      if (!active(operationId)) return;
      if (candidate === update) candidate = null;
      failedVersion = null;
      installedVersion = update.version;
      installed = true;
      state.set({
        status: "installing",
        currentVersion: options.currentVersion,
        nextVersion: update.version,
      });
      await gateway.relaunch();
    } catch (error) {
      if (!active(operationId)) return;
      if (installed) {
        state.set({
          status: "restart-required",
          currentVersion: options.currentVersion,
          nextVersion: update.version,
        });
        options.onRestartError?.(error);
        return;
      }

      if (candidate === update) candidate = null;
      failedVersion = update.version;
      await closeCandidate(update);
      if (!active(operationId)) return;
      state.set(failedState(update.version));
      onError?.(error);
    }
  }

  return {
    subscribe: state.subscribe,
    async check() {
      if (
        !options.nativeAvailable ||
        busy ||
        destroyed ||
        candidate !== null ||
        installedVersion !== null
      )
        return;
      const operationId = beginOperation();
      state.set({ status: "checking", currentVersion: options.currentVersion });
      try {
        const update = await gateway.check();
        if (!active(operationId)) {
          if (update !== null) await closeCandidate(update);
          return;
        }
        candidate = update;
        failedVersion = null;
        state.set(
          update === null
            ? { status: "idle", currentVersion: options.currentVersion }
            : availableState(update),
        );
      } catch (error) {
        console.error("LumaDesk update check failed", error);
        if (active(operationId)) {
          state.set({ status: "idle", currentVersion: options.currentVersion });
        }
      } finally {
        finishOperation(operationId);
      }
    },
    async install() {
      const update = candidate;
      if (update === null || busy || destroyed || installedVersion !== null) return;
      const operationId = beginOperation();
      try {
        await installCandidate(update, operationId, options.onInstallError);
      } finally {
        finishOperation(operationId);
      }
    },
    async retry() {
      const previousVersion = failedVersion;
      if (previousVersion === null || busy || destroyed || installedVersion !== null) return;
      const operationId = beginOperation();
      state.set({ status: "checking", currentVersion: options.currentVersion });
      try {
        let update: UpdateCandidate | null;
        try {
          update = await gateway.check();
        } catch (error) {
          if (!active(operationId)) return;
          state.set(failedState(previousVersion));
          options.onRetryError?.(error);
          return;
        }

        if (!active(operationId)) {
          if (update !== null) await closeCandidate(update);
          return;
        }
        if (update === null) {
          state.set(failedState(previousVersion));
          options.onRetryError?.(
            new Error("No compatible update package is available. Download it manually instead."),
          );
          return;
        }

        candidate = update;
        failedVersion = null;
        await installCandidate(update, operationId, options.onRetryError);
      } finally {
        finishOperation(operationId);
      }
    },
    async restart() {
      const nextVersion = installedVersion;
      if (nextVersion === null || busy || destroyed) return;
      const operationId = beginOperation();
      state.set({
        status: "installing",
        currentVersion: options.currentVersion,
        nextVersion,
      });
      try {
        await gateway.relaunch();
      } catch (error) {
        if (!active(operationId)) return;
        state.set({
          status: "restart-required",
          currentVersion: options.currentVersion,
          nextVersion,
        });
        options.onRestartError?.(error);
      } finally {
        finishOperation(operationId);
      }
    },
    destroy() {
      if (destroyed) return;
      destroyed = true;
      operation += 1;
      busy = false;
      const update = candidate;
      candidate = null;
      if (update !== null) void closeCandidate(update);
    },
  };
}

function assertNever(value: never): never {
  throw new TypeError(`Unexpected updater event: ${JSON.stringify(value)}`);
}
