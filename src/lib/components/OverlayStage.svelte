<script lang="ts">
import { onDestroy } from "svelte";
import type { OverlayInstance, OverlayPatch } from "../contracts";
import {
  type OverlayControlsHold,
  OverlayControlsVisibility,
} from "../overlay-controls-visibility";
import OverlayToolbar from "./OverlayToolbar.svelte";

export let canvas: HTMLCanvasElement;
export let mediaOpacity: number;
export let mediaTransform: string;
export let overlay: OverlayInstance | null;
export let busy: boolean;
export let error: string | null;
export let onstartdrag: (event: MouseEvent) => void;
export let onpatch: (patch: OverlayPatch) => void;
export let onduplicate: () => void;
export let onclose: () => void;

let controlsVisible = false;
const controlsVisibility = new OverlayControlsVisibility((visible) => {
  controlsVisible = visible;
});

onDestroy(() => controlsVisibility.dispose());

function handleMediaMouseDown(event: MouseEvent): void {
  controlsVisibility.reveal();
  onstartdrag(event);
}

function holdControls(reason: OverlayControlsHold): void {
  controlsVisibility.hold(reason);
}

function releaseControls(reason: OverlayControlsHold): void {
  controlsVisibility.release(reason);
}
</script>

<div class:controls-visible={controlsVisible} class="overlay-stage">
  <div class="media-stage" style:opacity={mediaOpacity} style:transform={mediaTransform} aria-hidden="true" onmousedown={handleMediaMouseDown}>
    <canvas bind:this={canvas}></canvas>
  </div>
  <OverlayToolbar
    {overlay}
    {busy}
    {error}
    visible={controlsVisible}
    {onstartdrag}
    {onpatch}
    {onduplicate}
    {onclose}
    onactivity={() => controlsVisibility.activity()}
    onhold={holdControls}
    onrelease={releaseControls}
  />
</div>

<style>
.overlay-stage {
  position: relative;
  width: 100vw;
  height: 100vh;
  background: transparent;
}
.overlay-stage::after {
  position: absolute;
  z-index: 1;
  inset: 0;
  pointer-events: none;
  box-shadow: inset 0 0 0 1px rgb(114 197 255 / 88%);
  content: "";
  opacity: 0;
  transition: opacity 140ms ease-out;
}
.overlay-stage.controls-visible::after { opacity: 1; }
.media-stage { width: 100%; height: 100%; cursor: move; transform-origin: center; }
canvas { display: block; width: 100%; height: 100%; }

@media (prefers-reduced-motion: reduce) {
  .overlay-stage::after { transition-duration: 120ms; }
}
</style>
