<script lang="ts">
  import { Archive, ArchiveRestore, FolderOpen, Trash2 } from "@lucide/svelte";
  import { Badge, IconButton } from "$lib/components/ui";
  import FactsGrid from "./FactsGrid.svelte";
  import TranscriptView from "./TranscriptView.svelte";
  import { t } from "$lib/i18n.svelte";
  import { workspaceSourceLabelKey } from "$lib/workspace";
  import type { CodexTranscript, CodexTranscriptMessage, Session } from "$lib/codex";
  import type { Workspace } from "$lib/workspace";

  type Props = {
    session: Session;
    workspace: Workspace | null;
    transcript: CodexTranscript | null;
    transcriptError: string;
    transcriptLoading: boolean;
    transcriptQuery: string;
    transcriptRoleFilter: string;
    transcriptRoleOptions: string[];
    filteredMessages: CodexTranscriptMessage[];
    filterActive: boolean;
    archiveBusy: boolean;
    trashBusy: boolean;
    onOpenWorkspace: () => void;
    onToggleArchive: () => void;
    onTrash: () => void;
    onTranscriptQueryChange: (value: string) => void;
    onTranscriptRoleChange: (value: string) => void;
    onClearTranscriptFilters: () => void;
  };

  let {
    session,
    workspace,
    transcript,
    transcriptError,
    transcriptLoading,
    transcriptQuery,
    transcriptRoleFilter,
    transcriptRoleOptions,
    filteredMessages,
    filterActive,
    archiveBusy,
    trashBusy,
    onOpenWorkspace,
    onToggleArchive,
    onTrash,
    onTranscriptQueryChange,
    onTranscriptRoleChange,
    onClearTranscriptFilters,
  }: Props = $props();

  const facts = $derived([
    {
      label: t("session.facts.status"),
      value: session.archived ? t("session.statusArchived") : t("session.statusActive"),
    },
    { label: t("session.facts.updated"), value: session.updatedAtLabel },
    {
      label: t("session.facts.workspace"),
      value: workspace?.name ?? t("common.unknown"),
      title: session.cwd,
    },
    {
      label: t("session.facts.source"),
      value: workspace ? t(workspaceSourceLabelKey(workspace.source)) : t("common.unknown"),
    },
    { label: t("session.facts.model"), value: session.model ?? t("common.unknown") },
    { label: t("session.facts.id"), value: session.id, mono: true },
    {
      label: t("session.facts.transcript"),
      value: session.rolloutPath,
      title: session.rolloutPath,
      mono: true,
      wide: true,
    },
  ]);
</script>

<header class="head">
  <div class="head-text">
    <span class="eyebrow">
      {#if session.archived}
        <Badge variant="warning">{t("session.statusArchived")}</Badge>
      {:else}
        <Badge variant="success">{t("session.statusActive")}</Badge>
      {/if}
      {#if session.model}
        <Badge variant="muted">{session.model}</Badge>
      {/if}
    </span>
    <h2 title={session.title}>{session.title}</h2>
    {#if workspace}
      <p title={workspace.path}>{workspace.path}</p>
    {/if}
  </div>

  <div class="head-actions">
    <IconButton
      aria-label={t("action.openWorkspace")}
      title={t("action.openWorkspace")}
      onclick={onOpenWorkspace}
    >
      <FolderOpen size={15} />
    </IconButton>
    <IconButton
      aria-label={session.archived ? t("action.restore") : t("action.archive")}
      title={session.archived ? t("action.restore") : t("action.archive")}
      disabled={archiveBusy}
      onclick={onToggleArchive}
    >
      {#if session.archived}
        <ArchiveRestore size={15} />
      {:else}
        <Archive size={15} />
      {/if}
    </IconButton>
    <IconButton
      variant="danger"
      aria-label={t("action.moveToTrash")}
      title={t("action.moveToTrash")}
      disabled={trashBusy}
      onclick={onTrash}
    >
      <Trash2 size={15} />
    </IconButton>
  </div>
</header>

<FactsGrid {facts} columns={2} />

<section class="card">
  <h3>{t("session.preview.title")}</h3>
  <p>{session.preview}</p>
</section>

<TranscriptView
  {transcript}
  error={transcriptError}
  loading={transcriptLoading}
  query={transcriptQuery}
  roleFilter={transcriptRoleFilter}
  roleOptions={transcriptRoleOptions}
  {filteredMessages}
  {filterActive}
  onQueryChange={onTranscriptQueryChange}
  onRoleChange={onTranscriptRoleChange}
  onClearFilters={onClearTranscriptFilters}
/>

<style>
  .head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .head-text {
    min-width: 0;
  }

  .eyebrow {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-bottom: 6px;
  }

  .head-text h2 {
    overflow: hidden;
    color: var(--fg);
    font-size: 18px;
    font-weight: 600;
    line-height: 1.3;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .head-text p {
    overflow: hidden;
    margin-top: 4px;
    color: var(--fg-tertiary);
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 16px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .head-actions {
    display: flex;
    flex-shrink: 0;
    gap: 4px;
  }

  .card {
    padding: 14px 16px;
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    border: 1px solid var(--separator);
  }

  .card h3 {
    margin-bottom: 6px;
    color: var(--fg-tertiary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .card p {
    color: var(--fg-secondary);
    font-size: 13px;
    line-height: 1.55;
  }
</style>
