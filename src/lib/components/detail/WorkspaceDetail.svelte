<script lang="ts">
  import { FolderOpen, Trash2 } from "@lucide/svelte";
  import { Badge, IconButton, Skeleton, StatePanel } from "$lib/components/ui";
  import FactsGrid from "./FactsGrid.svelte";
  import { formatCount, formatDate, formatSize } from "$lib/formatting";
  import {
    workspaceSourceDescriptionKey,
    workspaceSourceLabelKey,
    type Workspace,
  } from "$lib/workspace";
  import { t } from "$lib/i18n.svelte";
  import type { Session, WorkspaceMetadata } from "$lib/codex";

  type Props = {
    workspace: Workspace;
    metadata: WorkspaceMetadata | null;
    metadataError: string;
    metadataLoading: boolean;
    trashBusy: boolean;
    onOpenFolder: () => void;
    onTrash: () => void;
    onSelectSession: (session: Session) => void;
  };

  let {
    workspace,
    metadata,
    metadataError,
    metadataLoading,
    trashBusy,
    onOpenFolder,
    onTrash,
    onSelectSession,
  }: Props = $props();

  function badgeVariantFor(source: Workspace["source"]) {
    if (source === "codexTaskFolder") return "accent" as const;
    if (source === "codexWorktree") return "warning" as const;
    return "neutral" as const;
  }

  const sourceLabel = $derived(t(workspaceSourceLabelKey(workspace.source)));
  const sourceDescription = $derived(t(workspaceSourceDescriptionKey(workspace.source)));

  const summaryFacts = $derived([
    { label: t("workspaceDetail.facts.source"), value: sourceLabel },
    {
      label: t("workspaceDetail.facts.sessions"),
      value: t("workspaceDetail.facts.activeArchived", {
        active: formatCount(workspace.activeCount),
        archived: formatCount(workspace.archivedCount),
      }),
    },
    {
      label: t("workspaceDetail.facts.lastActivity"),
      value: workspace.updatedAtLabel,
    },
    {
      label: t("workspaceDetail.facts.path"),
      value: workspace.path,
      mono: true,
      title: workspace.path,
      wide: true,
    },
  ]);

  const metadataFacts = $derived(
    metadata
      ? [
          {
            label: t("settings.codexHome.facts.exists"),
            value: metadata.exists ? t("common.yes") : t("common.no"),
          },
          {
            label: t("workspaceDetail.facts.size"),
            value: formatSize(metadata.sizeBytes),
          },
          {
            label: t("workspaceDetail.facts.files"),
            value: formatCount(metadata.fileCount),
          },
          {
            label: t("workspaceDetail.facts.folders"),
            value: formatCount(metadata.directoryCount),
          },
          {
            label: t("session.facts.updated"),
            value: metadata.modifiedAtMs ? formatDate(metadata.modifiedAtMs) : t("common.unknown"),
          },
        ]
      : [],
  );
</script>

<header class="head">
  <div class="head-text">
    <span class="eyebrow">
      <Badge variant={badgeVariantFor(workspace.source)}>{sourceLabel}</Badge>
    </span>
    <h2 title={workspace.name}>{workspace.name}</h2>
    <p title={workspace.path}>{workspace.path}</p>
  </div>

  <div class="head-actions" aria-label={t("action.openWorkspace")}>
    <IconButton
      aria-label={t("action.openWorkspace")}
      title={t("action.openWorkspace")}
      onclick={onOpenFolder}
    >
      <FolderOpen size={15} />
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

<FactsGrid facts={summaryFacts} columns={2} />

<section class="card">
  <h3>{sourceLabel}</h3>
  <p>{sourceDescription}</p>
</section>

<section class="card">
  <header class="card-head">
    <h3>{t("workspaceDetail.facts.size")}</h3>
    <Badge variant="muted">
      {metadataLoading
        ? t("transcript.badge.loading")
        : metadata
          ? t("transcript.badge.onDemand")
          : t("common.loading")}
    </Badge>
  </header>

  {#if metadataError}
    <StatePanel variant="error">{metadataError}</StatePanel>
  {:else if metadata}
    <FactsGrid facts={metadataFacts} columns={3} />
    {#if metadata.scanTruncated}
      <p class="hint">{t("workspaceDetail.scanTruncated")}</p>
    {/if}
  {:else if metadataLoading}
    <div class="skeleton-grid">
      <Skeleton height="48px" radius="var(--radius-md)" />
      <Skeleton height="48px" radius="var(--radius-md)" />
      <Skeleton height="48px" radius="var(--radius-md)" />
      <Skeleton height="48px" radius="var(--radius-md)" />
      <Skeleton height="48px" radius="var(--radius-md)" />
      <Skeleton height="48px" radius="var(--radius-md)" />
    </div>
  {:else}
    <StatePanel variant="muted">{t("workspaceDetail.metadataLoading")}</StatePanel>
  {/if}
</section>

<section class="card">
  <header class="card-head">
    <h3>{t("workspaceDetail.relatedTitle")}</h3>
    <Badge variant="muted">{formatCount(workspace.sessionCount)}</Badge>
  </header>

  <div class="related-list">
    {#each workspace.sessions as session (session.id)}
      <button
        type="button"
        class="related"
        onclick={() => onSelectSession(session)}
      >
        <span class="related-title" title={session.title}>{session.title}</span>
        <span class="related-meta">
          <span>{session.updatedAtLabel}</span>
          <span class="dot" aria-hidden="true">·</span>
          <span>
            {session.archived
              ? t("session.statusArchived")
              : session.model ?? t("session.statusActive")}
          </span>
        </span>
      </button>
    {/each}
  </div>
</section>

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
    display: grid;
    gap: 10px;
    padding: 14px 16px;
    border-radius: var(--radius-md);
    background: var(--bg-surface);
    border: 1px solid var(--separator);
  }

  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .card h3 {
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

  .skeleton-grid {
    display: grid;
    gap: 6px;
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .hint {
    color: var(--fg-tertiary);
    font-size: 11px;
    line-height: 1.5;
  }

  .related-list {
    display: grid;
    gap: 4px;
  }

  .related {
    display: grid;
    gap: 2px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    background: transparent;
    border: 1px solid transparent;
    color: inherit;
    text-align: left;
    transition:
      background-color 60ms ease,
      border-color 60ms ease;
  }

  .related:hover {
    background: var(--bg-surface-hover);
    border-color: var(--separator);
  }

  .related-title {
    overflow: hidden;
    color: var(--fg);
    font-size: 13px;
    font-weight: 500;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .related-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--fg-tertiary);
    font-size: 11px;
  }

  .related-meta .dot {
    color: var(--fg-tertiary);
  }
</style>
