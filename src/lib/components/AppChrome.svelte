<script lang="ts">
import {
  Copy,
  Download,
  LoaderCircle,
  Minus,
  RefreshCw,
  Sparkles,
  Square,
  X,
} from "@lucide/svelte";
import { isTauri } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMount } from "svelte";
import packageMetadata from "../../../package.json";
import { createAppUpdater } from "../app-updater";
import { type NavigationId, navigation } from "../navigation";
import { userErrorMessage } from "../presentation";

export let activePage: NavigationId;
export let collectionCount = 0;
export let desktopCount = 0;
export let onnavigate: (id: NavigationId, enabled: boolean) => void;
export let onupdateerror: (message: string) => void;

const nativeWindow = isTauri() ? getCurrentWindow() : null;
const appUpdater = createAppUpdater({
  nativeAvailable: nativeWindow !== null,
  currentVersion: packageMetadata.version,
  onInstallError: (error) => {
    onupdateerror(userErrorMessage(error, "The update could not be installed. Try again."));
  },
  onRestartError: (error) => {
    onupdateerror(
      userErrorMessage(
        error,
        "The update is installed, but LumaDesk could not restart. Try again.",
      ),
    );
  },
});
let maximized = false;
let windowBusy = false;

onMount(() => {
  void appUpdater.check();
  if (nativeWindow === null) {
    return () => appUpdater.destroy();
  }
  let disposed = false;
  let unlisten: UnlistenFn | undefined;

  void nativeWindow.isMaximized().then((value) => {
    if (!disposed) maximized = value;
  });
  void nativeWindow
    .onResized(() => {
      void nativeWindow.isMaximized().then((value) => {
        if (!disposed) maximized = value;
      });
    })
    .then((cleanup) => {
      if (disposed) cleanup();
      else unlisten = cleanup;
    });

  return () => {
    disposed = true;
    unlisten?.();
    appUpdater.destroy();
  };
});

async function runWindowAction(action: () => Promise<void>): Promise<void> {
  if (nativeWindow === null || windowBusy) return;
  windowBusy = true;
  try {
    await action();
  } catch (error) {
    console.error("LumaDesk window action failed", error);
  } finally {
    windowBusy = false;
  }
}

async function toggleMaximize(): Promise<void> {
  if (nativeWindow === null) return;
  await runWindowAction(async () => {
    await nativeWindow.toggleMaximize();
    maximized = await nativeWindow.isMaximized();
  });
}
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <span class="brand-mark"><Sparkles size={16} strokeWidth={2.4} /></span>
    <span data-tauri-drag-region>LumaDesk</span>
  </div>
  <div class="titlebar-end">
    {#if $appUpdater.status === "available"}
      <button
        class="update-action"
        type="button"
        title={`Download and install LumaDesk ${$appUpdater.nextVersion}`}
        aria-label={`Download and install LumaDesk ${$appUpdater.nextVersion}`}
        onclick={() => void appUpdater.install()}
      >
        <Download size={14} />
        <span class="update-label">Update v{$appUpdater.nextVersion}</span>
      </button>
    {:else if $appUpdater.status === "downloading"}
      <span class="update-status" role="status" aria-live="polite">
        <LoaderCircle class="update-spinner" size={14} />
        <span class="update-label">{$appUpdater.progressPercent === null ? "Downloading" : `Downloading ${$appUpdater.progressPercent}%`}</span>
      </span>
    {:else if $appUpdater.status === "installing"}
      <span class="update-status" role="status" aria-live="polite">
        <LoaderCircle class="update-spinner" size={14} />
        <span class="update-label">Installing</span>
      </span>
    {:else if $appUpdater.status === "restart-required"}
      <button
        class="update-action"
        type="button"
        title={`Restart to finish installing LumaDesk ${$appUpdater.nextVersion}`}
        aria-label={`Restart to finish installing LumaDesk ${$appUpdater.nextVersion}`}
        onclick={() => void appUpdater.restart()}
      >
        <RefreshCw size={14} />
        <span class="update-label">Restart to finish update</span>
      </button>
    {:else}
      <span class="version" data-tauri-drag-region>v{packageMetadata.version}</span>
    {/if}
    {#if nativeWindow !== null}
      <div class="window-controls" aria-label="Window controls">
        <button type="button" aria-label="Minimize window" title="Minimize" disabled={windowBusy} onclick={() => void runWindowAction(() => nativeWindow.minimize())}><Minus size={15} /></button>
        <button type="button" aria-label={maximized ? "Restore window" : "Maximize window"} title={maximized ? "Restore" : "Maximize"} disabled={windowBusy} onclick={() => void toggleMaximize()}>
          {#if maximized}<Copy size={13} />{:else}<Square size={13} />{/if}
        </button>
        <button class="close-button" type="button" aria-label="Close window" title="Close" disabled={windowBusy} onclick={() => void runWindowAction(() => nativeWindow.close())}><X size={15} /></button>
      </div>
    {/if}
  </div>
</header>

<aside class="sidebar">
  <nav aria-label="Primary navigation">
    {#each navigation as item}
      <button
        class:active={activePage === item.id}
        type="button"
        disabled={!item.enabled}
        aria-current={activePage === item.id ? "page" : undefined}
        onclick={() => onnavigate(item.id, item.enabled)}
      >
        <item.icon size={18} />
        <span>{item.label}</span>
        {#if item.id === "collection"}
          <small>{collectionCount}</small>
        {:else if item.id === "desktop"}
          <small>{desktopCount}</small>
        {/if}
      </button>
    {/each}
  </nav>
</aside>

<style>
.titlebar {
  z-index: 10;
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--canvas-900);
  border-bottom: 1px solid var(--line-700);
  user-select: none;
}

.brand { display: flex; align-items: center; gap: 8px; font-family: "Space Grotesk Variable", sans-serif; font-weight: 680; }
.brand-mark { display: grid; width: 28px; height: 28px; place-items: center; color: var(--brand-300); background: var(--surface-800); border-radius: 6px; }
.version { color: var(--ink-500); font-size: 12px; }
.titlebar-end, .window-controls { display: flex; height: 100%; align-items: center; }
.titlebar-end { gap: 12px; }
.update-action, .update-status { display: flex; height: 28px; align-items: center; gap: 6px; padding: 0 9px; color: var(--brand-300); font-size: 12px; font-weight: 680; background: var(--surface-800); border: 1px solid var(--line-700); border-radius: 6px; }
.update-action { cursor: pointer; }
.update-action:hover { color: var(--ink-100); background: var(--surface-750); }
:global(.update-spinner) { animation: update-spin 900ms linear infinite; }
.window-controls { margin-right: -16px; }
.window-controls button { display: grid; width: 42px; height: 100%; padding: 0; place-items: center; color: var(--ink-300); background: transparent; border: 0; cursor: pointer; }
.window-controls button:hover { color: var(--ink-100); background: var(--surface-750); }
.window-controls button.close-button:hover { color: var(--ink-100); background: var(--danger-400); }
.window-controls button:disabled { cursor: default; opacity: 0.55; }

.sidebar {
  display: flex;
  min-height: 0;
  flex-direction: column;
  padding: 20px 12px 16px;
  background: var(--canvas-900);
  border-right: 1px solid var(--line-700);
}

nav { display: grid; gap: 4px; }
nav button {
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr) auto;
  width: 100%;
  min-height: 44px;
  align-items: center;
  gap: 12px;
  padding: 0 12px;
  color: var(--ink-300);
  text-align: left;
  background: transparent;
  border: 0;
  border-radius: 6px;
}
nav button.active { color: var(--ink-100); background: var(--surface-750); }
nav button:not(:disabled) { cursor: pointer; }
nav button:disabled { opacity: 0.52; }
nav small { color: var(--ink-500); font-size: 12px; }

@keyframes update-spin { to { transform: rotate(360deg); } }

@media (prefers-reduced-motion: reduce) {
  :global(.update-spinner) { animation: none; }
}

@media (max-width: 760px) {
  .sidebar { z-index: 8; grid-row: 3; padding: 8px 12px; border-top: 1px solid var(--line-700); border-right: 0; }
  nav { grid-template-columns: repeat(4, 1fr); }
  nav button { display: flex; flex-direction: column; justify-content: center; gap: 3px; padding: 0 4px; font-size: 12px; text-align: center; }
  nav button small { display: none; }
  .version { display: none; }
  .update-action, .update-status { width: 32px; justify-content: center; padding: 0; }
  .update-label { display: none; }
}
</style>
