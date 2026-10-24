<script lang="ts">
import { Sparkles } from "@lucide/svelte";
import { type NavigationId, navigation } from "../navigation";

export let activePage: NavigationId;
export let collectionCount = 0;
export let desktopCount = 0;
export let onnavigate: (id: NavigationId, enabled: boolean) => void;
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <span class="brand-mark"><Sparkles size={16} strokeWidth={2.4} /></span>
    <span data-tauri-drag-region>LumaDesk</span>
  </div>
  <span class="version" data-tauri-drag-region>v0.1.0</span>
</header>

<aside class="sidebar">
  <nav aria-label="Navigasi utama">
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
  <div class="privacy-note">
    <span class="status-dot"></span>
    <div><strong>Lokal sepenuhnya</strong><small>Tanpa akun atau telemetri</small></div>
  </div>
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

.sidebar {
  display: flex;
  min-height: 0;
  flex-direction: column;
  justify-content: space-between;
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

.privacy-note { display: flex; align-items: flex-start; gap: 10px; padding: 12px; color: var(--ink-300); font-size: 12px; background: var(--surface-850); border: 1px solid var(--line-700); border-radius: 8px; }
.privacy-note div { display: grid; gap: 3px; }
.privacy-note small { color: var(--ink-500); }
.status-dot { width: 8px; height: 8px; margin-top: 4px; background: var(--success-400); border-radius: 50%; }

@media (max-width: 760px) {
  .sidebar { z-index: 8; grid-row: 3; padding: 8px 12px; border-top: 1px solid var(--line-700); border-right: 0; }
  nav { grid-template-columns: repeat(3, 1fr); }
  nav button { display: flex; flex-direction: column; justify-content: center; gap: 3px; padding: 0 4px; font-size: 12px; text-align: center; }
  nav button small { display: none; }
  .privacy-note { display: none; }
}
</style>
