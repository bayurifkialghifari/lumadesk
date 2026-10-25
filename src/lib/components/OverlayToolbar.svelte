<script lang="ts">
import { Copy, Eraser, FlipHorizontal2, GripVertical, Pause, Play, X } from "@lucide/svelte";
import type { OverlayInstance, OverlayPatch } from "../contracts";
import type { OverlayControlsHold } from "../overlay-controls-visibility";

export let overlay: OverlayInstance | null;
export let busy = false;
export let error: string | null = null;
export let visible = false;
export let onstartdrag: (event: MouseEvent) => void;
export let onpatch: (patch: OverlayPatch) => void;
export let onduplicate: () => void;
export let onclose: () => void;
export let onactivity: () => void;
export let onhold: (reason: OverlayControlsHold) => void;
export let onrelease: (reason: OverlayControlsHold) => void;

let toolbar: HTMLDivElement;

function handleFocusOut(): void {
  queueMicrotask(() => {
    if (!toolbar.matches(":focus-within")) onrelease("focus");
  });
}
</script>

<div
  bind:this={toolbar}
  class:visible
  class="overlay-toolbar"
  role="toolbar"
  aria-label="Overlay controls"
  aria-hidden={!visible}
  onpointerenter={() => onhold("pointer")}
  onpointerleave={() => onrelease("pointer")}
  onpointerdown={onactivity}
  onfocusin={() => onhold("focus")}
  onfocusout={handleFocusOut}
  onkeydown={onactivity}
>
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
  <button
    class:active={overlay?.removeBackground === true}
    type="button"
    aria-label={overlay?.removeBackground === true ? "Restore background" : "Remove background"}
    aria-pressed={overlay?.removeBackground === true}
    title={overlay?.removeBackground === true ? "Restore background" : "Remove background"}
    disabled={busy || overlay === null}
    onclick={() => onpatch({ removeBackground: overlay?.removeBackground !== true })}
  >
    <Eraser size={15} />
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
  grid-template-columns: repeat(6, 30px);
  gap: 2px;
  padding: 3px;
  background: rgb(8 17 28 / 92%);
  border: 1px solid rgb(114 197 255 / 52%);
  border-radius: 6px;
  box-shadow: 0 6px 18px rgb(0 0 0 / 35%);
  visibility: hidden;
  opacity: 0;
  pointer-events: none;
  transform: translate(-50%, -4px);
  transition:
    opacity 140ms ease-out,
    transform 140ms ease-out,
    visibility 0s linear 140ms;
  backdrop-filter: blur(8px);
}

.overlay-toolbar.visible {
  visibility: visible;
  opacity: 1;
  pointer-events: auto;
  transform: translate(-50%, 0);
  transition-delay: 0s;
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
.overlay-toolbar button.active { color: #07111d; background: #72c5ff; }
.overlay-toolbar button.active:hover { color: #07111d; background: #9bd8ff; }
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

@media (prefers-reduced-motion: reduce) {
  .overlay-toolbar {
    transform: translateX(-50%);
    transition:
      opacity 120ms ease-out,
      visibility 0s linear 120ms;
  }
  .overlay-toolbar.visible {
    transform: translateX(-50%);
    transition-delay: 0s;
  }
}
</style>
