<script lang="ts">
  import { onMount } from "svelte";
  import { FileText, RefreshCw, Save } from "@lucide/svelte";
  import { Badge, Button, Dialog, StatePanel } from "$lib/components/ui";
  import {
    getGlobalAgentDocument,
    getWorkspaceAgentDocument,
    saveGlobalAgentDocument,
    saveWorkspaceAgentDocument,
    type CodexAgentDocument,
  } from "$lib/codex";
  import { formatDate, formatSize } from "$lib/formatting";
  import { t } from "$lib/i18n.svelte";
  import { toast } from "$lib/toast.svelte";

  type AgentDocumentMode = "workspace" | "global";

  type Props = {
    mode?: AgentDocumentMode;
    workspacePath?: string;
  };

  let { mode = "workspace", workspacePath = "" }: Props = $props();

  let agentDocument = $state<CodexAgentDocument | null>(null);
  let draftContent = $state("");
  let loading = $state(false);
  let saving = $state(false);
  let error = $state("");
  let saveDialogOpen = $state(false);

  const dirty = $derived(agentDocument ? draftContent !== agentDocument.content : false);
  const canSave = $derived(Boolean(agentDocument) && (dirty || agentDocument?.exists === false));
  const modifiedLabel = $derived(
    agentDocument?.modifiedAtMs ? formatDate(agentDocument.modifiedAtMs) : t("common.unknown"),
  );
  const titleKey = $derived(
    mode === "global" ? "agentDocument.globalTitle" : "agentDocument.title",
  );
  const descriptionKey = $derived(
    mode === "global" ? "agentDocument.globalDescription" : "agentDocument.description",
  );
  const placeholderKey = $derived(
    mode === "global" ? "agentDocument.globalPlaceholder" : "agentDocument.placeholder",
  );
  const emptyHintKey = $derived(
    mode === "global" ? "agentDocument.globalEmptyHint" : "agentDocument.emptyHint",
  );
  const createKey = $derived(
    mode === "global" ? "agentDocument.globalCreate" : "agentDocument.create",
  );
  const saveKey = $derived(
    mode === "global" ? "agentDocument.globalSave" : "agentDocument.save",
  );
  const dialogTitleKey = $derived(
    mode === "global" ? "dialog.globalAgentDocumentSave.title" : "dialog.agentDocumentSave.title",
  );
  const dialogBodyKey = $derived(
    mode === "global" ? "dialog.globalAgentDocumentSave.body" : "dialog.agentDocumentSave.body",
  );
  const dialogNoteKey = $derived(
    mode === "global" ? "dialog.globalAgentDocumentSave.note" : "dialog.agentDocumentSave.note",
  );

  function errorMessage(value: unknown) {
    return value instanceof Error ? value.message : String(value);
  }

  function applyDocument(next: CodexAgentDocument) {
    agentDocument = next;
    draftContent = next.content;
  }

  async function loadDocument() {
    loading = true;
    error = "";
    try {
      applyDocument(
        mode === "global" ? await getGlobalAgentDocument() : await getWorkspaceAgentDocument(workspacePath),
      );
    } catch (loadError) {
      error = errorMessage(loadError);
    } finally {
      loading = false;
    }
  }

  async function saveDocument() {
    if (!agentDocument) return;

    saving = true;
    error = "";
    try {
      applyDocument(
        mode === "global"
          ? await saveGlobalAgentDocument({
              revision: agentDocument.revision,
              content: draftContent,
            })
          : await saveWorkspaceAgentDocument({
              workspacePath,
              revision: agentDocument.revision,
              content: draftContent,
            }),
      );
      saveDialogOpen = false;
      toast.success(
        t(mode === "global" ? "toast.globalAgentDocumentSaved" : "toast.agentDocumentSaved"),
      );
    } catch (saveError) {
      error = errorMessage(saveError);
      toast.error(
        t(mode === "global" ? "toast.globalAgentDocumentFailed" : "toast.agentDocumentFailed"),
        error,
      );
    } finally {
      saving = false;
    }
  }

  onMount(() => {
    void loadDocument();
  });
</script>

<section class="card">
  <header class="card-head">
    <div class="title-row">
      <FileText size={14} />
      <h3>{t(titleKey)}</h3>
    </div>
    <div class="head-actions">
      {#if loading && !agentDocument}
        <Badge variant="muted">{t("common.loading")}</Badge>
      {:else if agentDocument?.exists}
        <Badge variant="success">{t("agentDocument.exists")}</Badge>
      {:else}
        <Badge variant="warning">{t("agentDocument.missing")}</Badge>
      {/if}
      <Button variant="ghost" size="sm" loading={loading} onclick={loadDocument}>
        <RefreshCw size={13} />
        {t("agentDocument.reload")}
      </Button>
    </div>
  </header>

  <p class="card-text">{t(descriptionKey)}</p>

  {#if error}
    <StatePanel variant="error">{error}</StatePanel>
  {/if}

  {#if agentDocument}
    <div class="doc-facts">
      <span title={agentDocument.path}>{t("agentDocument.path")}: {agentDocument.path}</span>
      <span>{t("agentDocument.size")}: {formatSize(agentDocument.sizeBytes)}</span>
      <span>{t("agentDocument.modified")}: {modifiedLabel}</span>
    </div>

    <textarea
      bind:value={draftContent}
      class="editor"
      spellcheck="false"
      placeholder={t(placeholderKey)}
      aria-label={t(titleKey)}
    ></textarea>

    <div class="actions">
      {#if dirty}
        <span class="dirty-label">{t("agentDocument.unsaved")}</span>
      {:else if !agentDocument.exists}
        <span class="dirty-label">{t(emptyHintKey)}</span>
      {/if}
      <Button
        variant="primary"
        size="sm"
        loading={saving}
        disabled={!canSave}
        onclick={() => (saveDialogOpen = true)}
      >
        <Save size={13} />
        {agentDocument.exists ? t(saveKey) : t(createKey)}
      </Button>
    </div>
  {:else if loading}
    <StatePanel variant="muted">{t("common.loading")}</StatePanel>
  {/if}
</section>

<Dialog
  bind:open={saveDialogOpen}
  title={t(dialogTitleKey)}
  description={agentDocument
    ? t(dialogBodyKey, { path: agentDocument.path })
    : t(descriptionKey)}
  onclose={() => (saveDialogOpen = false)}
>
  {#if agentDocument}
    <p class="dialog-target" title={agentDocument.path}>{agentDocument.path}</p>
    <p class="dialog-note">{t(dialogNoteKey)}</p>
  {/if}

  {#snippet actions()}
    <Button variant="secondary" onclick={() => (saveDialogOpen = false)}>
      {t("common.cancel")}
    </Button>
    <Button variant="primary" loading={saving} onclick={saveDocument}>
      {agentDocument?.exists ? t(saveKey) : t(createKey)}
    </Button>
  {/snippet}
</Dialog>

<style>
  .card {
    display: grid;
    gap: 10px;
    padding: 14px 16px;
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    border: 1px solid var(--separator);
  }

  .card-head,
  .title-row,
  .head-actions,
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .card-head {
    justify-content: space-between;
  }

  .title-row {
    color: var(--fg-tertiary);
  }

  .card h3 {
    color: var(--fg-tertiary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .card-text {
    color: var(--fg-secondary);
    font-size: 13px;
    line-height: 1.55;
  }

  .doc-facts {
    display: grid;
    gap: 4px;
    color: var(--fg-tertiary);
    font-size: 11px;
    line-height: 1.45;
  }

  .doc-facts span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .doc-facts span:first-child {
    font-family: var(--font-mono);
  }

  .editor {
    width: 100%;
    min-height: 220px;
    resize: vertical;
    padding: 10px 11px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--fg);
    background: var(--bg-input);
    box-shadow: var(--shadow-sm);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.55;
  }

  .editor::placeholder {
    color: var(--fg-tertiary);
  }

  .editor:focus-visible {
    outline: 2px solid var(--ring);
    outline-offset: -1px;
    border-color: var(--accent);
  }

  .actions {
    justify-content: space-between;
  }

  .dirty-label {
    color: var(--fg-tertiary);
    font-size: 12px;
  }

  @media (max-width: 640px) {
    .card-head,
    .actions {
      align-items: stretch;
      flex-direction: column;
    }

    .head-actions,
    .actions :global(.btn) {
      width: 100%;
    }
  }
</style>
