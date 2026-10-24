<script lang="ts">
import { AlertTriangle, CheckCircle2, Info, X } from "@lucide/svelte";
import type { ToastTone } from "../collection-model";

export let message: string;
export let tone: ToastTone;
export let ondismiss: () => void;
</script>

<div class:error={tone === "error"} class:success={tone === "success"} class="toast" role={tone === "error" ? "alert" : "status"}>
  <span class="toast__icon" aria-hidden="true">
    {#if tone === "success"}
      <CheckCircle2 size={18} />
    {:else if tone === "error"}
      <AlertTriangle size={18} />
    {:else}
      <Info size={18} />
    {/if}
  </span>
  <p>{message}</p>
  <button type="button" aria-label="Tutup pemberitahuan" onclick={ondismiss}><X size={16} /></button>
</div>

<style>
.toast {
  position: fixed;
  z-index: 40;
  right: 20px;
  bottom: 20px;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  width: min(380px, calc(100% - 32px));
  align-items: center;
  gap: 12px;
  padding: 12px;
  color: var(--ink-100);
  background: var(--surface-750);
  border: 1px solid var(--line-700);
  border-radius: 8px;
  box-shadow: 0 12px 36px color-mix(in srgb, var(--canvas-950) 72%, transparent);
  animation: toast-enter 180ms ease-out;
}

.toast__icon {
  display: grid;
  width: 32px;
  height: 32px;
  place-items: center;
  color: var(--brand-300);
  background: var(--surface-800);
  border-radius: 6px;
}

.toast.success .toast__icon {
  color: var(--success-400);
}

.toast.error .toast__icon {
  color: var(--danger-400);
}

p {
  margin: 0;
  overflow-wrap: anywhere;
  font-size: 13px;
}

button {
  display: grid;
  width: 36px;
  height: 36px;
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

@keyframes toast-enter {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 760px) {
  .toast {
    right: 16px;
    bottom: 80px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .toast {
    animation: toast-fade 100ms linear;
  }

  @keyframes toast-fade {
    from { opacity: 0; }
    to { opacity: 1; }
  }
}
</style>
