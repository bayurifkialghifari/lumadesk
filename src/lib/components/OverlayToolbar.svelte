<script lang="ts">
import { Copy, FlipHorizontal2, GripVertical, Pause, Play, X } from "@lucide/svelte";
import type { OverlayInstance, OverlayPatch } from "../contracts";

export let overlay: OverlayInstance | null;
export let busy = false;
export let error: string | null = null;
export let onstartdrag: (event: MouseEvent) => void;
export let onpatch: (patch: OverlayPatch) => void;
export let onduplicate: () => void;
export let onclose: () => void;
</script>

<div class="overlay-toolbar" role="toolbar" aria-label="Overlay controls">
  <button type="button" aria-label="Move overlay" title="Move" disabled={busy} onmousedown={onstartdrag}>
    <GripVertical size={15} />
  </button>
  <button
    type="button"
    aria-label={overlay?.isPlaying === true ? "Pause animation" : "Play animation"}
    title={overlay?.isPlaying === true ? "Pause" : "Play"}
    disabled={busy || overlay === null}
    onclick={() => onpatch({ isPlaying: overlay?.isPlaying !== true })}
  >
    {#if overlay?.isPlaying === true}<Pause size={15} />{:else}<Play size={15} />{/if}
  </button>
  <button
    type="button"
    aria-label="Flip horizontally"
    title="Flip horizontally"
    disabled={busy || overlay === null}
    onclick={() => onpatch({ flipHorizontal: overlay?.flipHorizontal !== true })}
  >
    <FlipHorizontal2 size={15} />
  </button>
  <button type="button" aria-label="Duplicate overlay" title="Duplicate" disabled={busy || overlay === null} onclick={onduplicate}>
    <Copy size={15} />
  </button>
  <button class="danger" type="button" aria-label="Close overlay" title="Close" disabled={busy || overlay === null} onclick={onclose}>
    <X size={15} />
  </button>
</div>

{#if error !== null}<div class="overlay-error" role="alert">{error}</div>{/if}

<style>
.overlay-toolbar {
  position: absolute;
  z-index: 2;
  top: 5px;
  left: 50%;
  display: grid;
  grid-template-columns: repeat(5, 30px);
  gap: 2px;
  padding: 3px;
  background: rgb(8 17 28 / 92%);
  border: 1px solid rgb(114 197 255 / 52%);
  border-radius: 6px;
  box-shadow: 0 6px 18px rgb(0 0 0 / 35%);
  transform: translateX(-50%);
  backdrop-filter: blur(8px);
}

.overlay-toolbar button {
  display: grid;
  width: 30px;
  height: 30px;
  padding: 0;
  place-items: center;
  color: #b8c7d9;
  background: transparent;
  border: 0;
  border-radius: 4px;
  cursor: pointer;
}

.overlay-toolbar button:hover { color: #f4f8fc; background: #1e354e; }
.overlay-toolbar button.danger:hover { color: #ff6b75; }
.overlay-toolbar button:disabled { cursor: wait; opacity: 0.5; }

.overlay-error {
  position: absolute;
  z-index: 3;
  right: 5px;
  bottom: 5px;
  left: 5px;
  overflow: hidden;
  padding: 6px 8px;
  color: #fff;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
  background: rgb(115 23 33 / 94%);
  border-radius: 4px;
}
</style>
