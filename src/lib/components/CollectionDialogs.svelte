<script lang="ts">
import { AlertTriangle, CheckCircle2, Copy, FileWarning, Tag } from "@lucide/svelte";
import type { CollectionDialog } from "../collection-model";
import type { ImportReport, MediaItem } from "../contracts";
import { filenameFromPath, parseTags } from "../presentation";
import Modal from "./Modal.svelte";

export let dialog: CollectionDialog | null;
export let report: ImportReport | null;
export let busy = false;
export let onclose: () => void;
export let onclosereport: () => void;
export let onrename: (item: MediaItem, name: string) => Promise<boolean>;
export let ontags: (item: MediaItem, tags: readonly string[]) => Promise<boolean>;
export let ondelete: (item: MediaItem) => Promise<boolean>;

let activeDialogKey = "";
let name = "";
let tags = "";

$: dialogKey = dialog === null ? "" : `${dialog.kind}:${dialog.item.id}`;
$: if (dialogKey !== activeDialogKey) {
  activeDialogKey = dialogKey;
  if (dialog !== null) {
    name = dialog.item.displayName;
    tags = dialog.item.tags.join(", ");
  }
}

async function submitRename(event: SubmitEvent): Promise<void> {
  event.preventDefault();
  if (dialog?.kind !== "rename") return;
  if (await onrename(dialog.item, name)) onclose();
}

async function submitTags(event: SubmitEvent): Promise<void> {
  event.preventDefault();
  if (dialog?.kind !== "tags") return;
  if (await ontags(dialog.item, parseTags(tags))) onclose();
}

async function submitDelete(): Promise<void> {
  if (dialog?.kind !== "delete") return;
  if (await ondelete(dialog.item)) onclose();
}
</script>

{#if dialog?.kind === "rename"}
  <Modal title="Rename animation" description="Change the catalog name without renaming the managed file." {busy} {onclose}>
    <form onsubmit={submitRename}>
      <label for="rename-media">Display name</label>
      <input id="rename-media" bind:value={name} maxlength="120" required />
      <div class="dialog-actions">
        <button class="secondary-button" type="button" disabled={busy} onclick={onclose}>Cancel</button>
        <button class="primary-button" type="submit" disabled={busy || name.trim() === ""}>Save name</button>
      </div>
    </form>
  </Modal>
{:else if dialog?.kind === "tags"}
  <Modal title="Manage tags" description="Separate multiple tags with commas." {busy} {onclose}>
    <form onsubmit={submitTags}>
      <label for="media-tags"><Tag size={15} /> Animation tags</label>
      <input id="media-tags" bind:value={tags} placeholder="pixel, ambient, character" />
      <p class="field-help">Duplicate tags and extra spaces are cleaned up automatically.</p>
      <div class="dialog-actions">
        <button class="secondary-button" type="button" disabled={busy} onclick={onclose}>Cancel</button>
        <button class="primary-button" type="submit" disabled={busy}>Save tags</button>
      </div>
    </form>
  </Modal>
{:else if dialog?.kind === "delete"}
  <Modal title="Remove from collection?" description={`“${dialog.item.displayName}” will be removed from LumaDesk.`} {busy} {onclose}>
    <div class="delete-copy">
      <span><AlertTriangle size={20} /></span>
      <p>The managed copy, cache, and related overlays will be removed. The original source file stays untouched.</p>
    </div>
    <div class="dialog-actions">
      <button class="secondary-button" type="button" disabled={busy} onclick={onclose}>Cancel</button>
      <button class="danger-button" type="button" disabled={busy} onclick={submitDelete}>Remove animation</button>
    </div>
  </Modal>
{:else if report !== null}
  <Modal title="Import results" description="Each file is processed independently so one failure does not stop the rest." onclose={onclosereport}>
    <div class="report-summary">
      <span><CheckCircle2 size={17} /> {report.outcomes.filter((outcome) => outcome.status === "imported").length} new</span>
      <span><Copy size={17} /> {report.outcomes.filter((outcome) => outcome.status === "duplicate").length} duplicate</span>
      <span class:error={report.failures.length > 0}><FileWarning size={17} /> {report.failures.length} failed</span>
    </div>
    <div class="report-list">
      {#each report.outcomes as outcome}
        <div><span class="result-icon success"><CheckCircle2 size={16} /></span><strong>{outcome.item.displayName}</strong><small>{outcome.status === "imported" ? "Added" : "Already exists"}</small></div>
      {/each}
      {#each report.failures as failure}
        <div><span class="result-icon failure"><FileWarning size={16} /></span><strong>{filenameFromPath(failure.path)}</strong><small>{failure.message}</small></div>
      {/each}
    </div>
    <div class="dialog-actions"><button class="primary-button" type="button" onclick={onclosereport}>Done</button></div>
  </Modal>
{/if}

<style>
form, .report-list { display: grid; gap: 12px; }
form, .delete-copy, .report-summary, .report-list, .dialog-actions { padding-inline: 20px; }
form { padding-block: 20px; }
label { display: flex; align-items: center; gap: 8px; color: var(--ink-300); font-size: 13px; font-weight: 650; }
input {
  width: 100%;
  min-height: 40px;
  padding-inline: 12px;
  color: var(--ink-100);
  background: var(--canvas-950);
  border: 1px solid var(--line-700);
  border-radius: 6px;
  outline: 0;
}
input:focus { border-color: var(--brand-400); box-shadow: 0 0 0 2px color-mix(in srgb, var(--brand-400) 24%, transparent); }
.field-help { margin: -4px 0 0; color: var(--ink-500); font-size: 12px; }

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-block: 16px;
  border-top: 1px solid var(--line-700);
}
form .dialog-actions { margin: 8px -20px -20px; }

.primary-button, .secondary-button, .danger-button {
  display: inline-flex;
  min-height: 40px;
  align-items: center;
  justify-content: center;
  padding-inline: 16px;
  font-weight: 650;
  border-radius: 6px;
  cursor: pointer;
}
.primary-button { color: var(--canvas-950); background: var(--brand-400); border: 0; }
.secondary-button { color: var(--ink-300); background: var(--surface-800); border: 1px solid var(--line-700); }
.danger-button { color: var(--ink-100); background: var(--danger-400); border: 0; }
button:disabled { cursor: wait; opacity: 0.55; }

.delete-copy { display: flex; gap: 12px; padding-block: 20px; }
.delete-copy span { display: grid; width: 40px; height: 40px; flex: 0 0 auto; place-items: center; color: var(--danger-400); background: color-mix(in srgb, var(--danger-400) 10%, var(--surface-800)); border-radius: 8px; }
.delete-copy p { margin: 0; color: var(--ink-300); font-size: 13px; }

.report-summary { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; padding-block: 16px; }
.report-summary span { display: flex; align-items: center; gap: 8px; padding: 10px; color: var(--success-400); font-size: 12px; background: var(--surface-800); border-radius: 6px; }
.report-summary span.error { color: var(--danger-400); }
.report-list { max-height: 280px; overflow: auto; padding-bottom: 16px; }
.report-list > div { display: grid; grid-template-columns: 32px minmax(0, 1fr); gap: 2px 8px; align-items: center; }
.report-list strong { overflow: hidden; font-size: 13px; text-overflow: ellipsis; white-space: nowrap; }
.report-list small { grid-column: 2; color: var(--ink-500); font-size: 12px; }
.result-icon { display: grid; grid-row: 1 / 3; width: 32px; height: 32px; place-items: center; background: var(--surface-800); border-radius: 6px; }
.result-icon.success { color: var(--success-400); }
.result-icon.failure { color: var(--danger-400); }

@media (max-width: 480px) {
  .report-summary { grid-template-columns: 1fr; }
  .dialog-actions { display: grid; grid-template-columns: 1fr 1fr; }
}
</style>
