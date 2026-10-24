<script lang="ts">
import type { OverlayInstance, OverlayPatch } from "../contracts";
import OverlayToolbar from "./OverlayToolbar.svelte";

export let canvas: HTMLCanvasElement;
export let editMode: boolean;
export let mediaOpacity: number;
export let mediaTransform: string;
export let overlay: OverlayInstance | null;
export let busy: boolean;
export let error: string | null;
export let onstartdrag: (event: MouseEvent) => void;
export let onpatch: (patch: OverlayPatch) => void;
export let onduplicate: () => void;
export let onclose: () => void;
</script>

<div class:editing={editMode} class="overlay-stage">
  <div class="media-stage" style:opacity={mediaOpacity} style:transform={mediaTransform} aria-hidden="true">
    <canvas bind:this={canvas}></canvas>
  </div>

  {#if editMode}
    <OverlayToolbar {overlay} {busy} {error} {onstartdrag} {onpatch} {onduplicate} {onclose} />
  {/if}
</div>

<style>
.overlay-stage {
  position: relative;
  width: 100vw;
  height: 100vh;
  background: transparent;
}

.overlay-stage.editing { box-shadow: inset 0 0 0 1px rgb(114 197 255 / 88%); }
.media-stage { width: 100%; height: 100%; transform-origin: center; }
canvas { display: block; width: 100%; height: 100%; }
</style>
