<script lang="ts">
import { X } from "@lucide/svelte";
import { onMount } from "svelte";

export let title: string;
export let description: string;
export let busy = false;
export let onclose: () => void;

let dialog: HTMLDialogElement;

onMount(() => {
  dialog.showModal();
  return () => dialog.close();
});

function handleCancel(event: Event): void {
  if (busy) {
    event.preventDefault();
    return;
  }
  onclose();
}

function handleBackdrop(event: MouseEvent): void {
  if (event.target === dialog && !busy) {
    onclose();
  }
}
</script>

<dialog bind:this={dialog} aria-labelledby="action-dialog-title" aria-describedby="action-dialog-copy" oncancel={handleCancel} onclick={handleBackdrop}>
  <section class="dialog-panel">
    <header>
      <div>
        <h2 id="action-dialog-title">{title}</h2>
        <p id="action-dialog-copy">{description}</p>
      </div>
      <button type="button" aria-label="Tutup dialog" disabled={busy} onclick={onclose}>
        <X size={18} />
      </button>
    </header>
    <slot></slot>
  </section>
</dialog>

<style>
dialog {
  width: min(440px, calc(100% - 32px));
  max-height: calc(100dvh - 32px);
  padding: 0;
  overflow: visible;
  color: var(--ink-100);
  background: transparent;
  border: 0;
}

dialog::backdrop {
  background: color-mix(in srgb, var(--canvas-950) 76%, transparent);
}

.dialog-panel {
  overflow: hidden;
  background: var(--surface-850);
  border: 1px solid var(--line-700);
  border-radius: 12px;
  box-shadow: 0 12px 36px color-mix(in srgb, var(--canvas-950) 72%, transparent);
  animation: dialog-enter 180ms ease-out;
}

header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 20px 20px 16px;
  border-bottom: 1px solid var(--line-700);
}

h2 {
  margin: 0 0 4px;
  font-size: 20px;
}

p {
  margin: 0;
  color: var(--ink-300);
  font-size: 13px;
}

button {
  display: grid;
  width: 40px;
  height: 40px;
  flex: 0 0 auto;
  place-items: center;
  color: var(--ink-300);
  background: transparent;
  border: 0;
  border-radius: 6px;
  cursor: pointer;
}

button:hover {
  color: var(--ink-100);
  background: var(--surface-800);
}

button:disabled {
  cursor: wait;
  opacity: 0.55;
}

@keyframes dialog-enter {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .dialog-panel {
    animation: dialog-fade 100ms linear;
  }

  @keyframes dialog-fade {
    from { opacity: 0; }
    to { opacity: 1; }
  }
}
</style>
