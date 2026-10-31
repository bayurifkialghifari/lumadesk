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
  restart(): Promise<void>;
  destroy(): void;
}

interface AppUpdaterOptions {
  readonly nativeAvailable: boolean;
  readonly currentVersion: string;
  readonly gateway?: UpdaterGateway;
  readonly onInstallError?: (error: unknown) => void;
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
  let candidate: UpdateCandidate | null = null;
  let installedVersion: string | null = null;
  let busy = false;
  let destroyed = false;
  let downloadedBytes = 0;
  let totalBytes: number | undefined;

  function availableState(update: UpdateCandidate): AppUpdaterState {
    return {
      status: "available",
      currentVersion: options.currentVersion,
      nextVersion: update.version,
    };
  }

  function handleDownload(event: DownloadEvent, update: UpdateCandidate): void {
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
      busy = true;
      state.set({ status: "checking", currentVersion: options.currentVersion });
      try {
        const update = await gateway.check();
        if (destroyed) {
          if (update !== null) await update.close();
          return;
        }
        candidate = update;
        state.set(
          update === null
            ? { status: "idle", currentVersion: options.currentVersion }
            : availableState(update),
        );
      } catch (error) {
        console.error("LumaDesk update check failed", error);
        state.set({ status: "idle", currentVersion: options.currentVersion });
      } finally {
        busy = false;
      }
    },
    async install() {
      const update = candidate;
      if (update === null || busy || destroyed) return;
      busy = true;
      state.set({
        status: "downloading",
        currentVersion: options.currentVersion,
        nextVersion: update.version,
        progressPercent: null,
      });
      let installed = false;
      try {
        await update.downloadAndInstall((event) => handleDownload(event, update));
        candidate = null;
        installedVersion = update.version;
        installed = true;
        state.set({
          status: "installing",
          currentVersion: options.currentVersion,
          nextVersion: update.version,
        });
        await gateway.relaunch();
      } catch (error) {
        if (installed) {
          state.set({
            status: "restart-required",
            currentVersion: options.currentVersion,
            nextVersion: update.version,
          });
          options.onRestartError?.(error);
        } else {
          state.set(availableState(update));
          options.onInstallError?.(error);
        }
      } finally {
        busy = false;
      }
    },
    async restart() {
      const nextVersion = installedVersion;
      if (nextVersion === null || busy || destroyed) return;
      busy = true;
      state.set({
        status: "installing",
        currentVersion: options.currentVersion,
        nextVersion,
      });
      try {
        await gateway.relaunch();
      } catch (error) {
        state.set({
          status: "restart-required",
          currentVersion: options.currentVersion,
          nextVersion,
        });
        options.onRestartError?.(error);
      } finally {
        busy = false;
      }
    },
    destroy() {
      destroyed = true;
      const update = candidate;
      candidate = null;
      if (update !== null) {
        void update.close().catch((error: unknown) => {
          console.error("LumaDesk updater cleanup failed", error);
        });
      }
    },
  };
}

function assertNever(value: never): never {
  throw new TypeError(`Unexpected updater event: ${JSON.stringify(value)}`);
}
