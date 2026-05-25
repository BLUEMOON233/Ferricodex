<script lang="ts">
  import { ChevronDown, ChevronRight, Folder, FolderGit2, Sparkles } from "@lucide/svelte";
  import SessionRow from "./SessionRow.svelte";
  import { collapse } from "$lib/collapse.svelte";
  import { t } from "$lib/i18n.svelte";
  import type { Session } from "$lib/codex";
  import type { Workspace, WorkspaceSource } from "$lib/workspace";

  type WorkspaceListItem = Workspace & {
    visibleSessions: Session[];
    visibleActiveCount: number;
    visibleArchivedCount: number;
  };

  type Props = {
    workspace: WorkspaceListItem;
    selectedSessionId: string;
    selectedWorkspaceId: string;
    selectionKind: "session" | "workspace";
    selectedSessionIds: string[];
    forceExpanded?: boolean;
    onSelectWorkspace: (workspace: Workspace) => void;
    onSelectSession: (session: Session) => void;
    onToggleSession: (session: Session, checked: boolean) => void;
  };

  let {
    workspace,
    selectedSessionId,
    selectedWorkspaceId,
    selectionKind,
    selectedSessionIds,
    forceExpanded = false,
    onSelectWorkspace,
    onSelectSession,
    onToggleSession,
  }: Props = $props();

  const isWorkspaceActive = $derived(
    selectionKind === "workspace" && selectedWorkspaceId === workspace.id,
  );

  const collapsed = $derived(!forceExpanded && collapse.isWorkspaceCollapsed(workspace.id));

  function iconFor(source: WorkspaceSource) {
    if (source === "codexTaskFolder") return Sparkles;
    if (source === "codexWorktree") return FolderGit2;
    return Folder;
  }

  const SourceIcon = $derived(iconFor(workspace.source));
</script>

<article class="workspace-card" class:active={isWorkspaceActive}>
  <div class="workspace-row">
    <button
      type="button"
      class="disclosure"
      aria-expanded={!collapsed}
      aria-label={collapsed
        ? t("workspace.toggle.expand", { name: workspace.name })
        : t("workspace.toggle.collapse", { name: workspace.name })}
      onclick={() => collapse.toggleWorkspace(workspace.id)}
    >
      {#if collapsed}
        <ChevronRight size={12} />
      {:else}
        <ChevronDown size={12} />
      {/if}
    </button>

    <button
      type="button"
      class="workspace-header"
      onclick={() => onSelectWorkspace(workspace)}
    >
      <span class="header-icon" aria-hidden="true">
        <SourceIcon size={14} />
      </span>
      <span class="header-text">
        <span class="header-title">
          <span class="workspace-name" title={workspace.name}>{workspace.name}</span>
        </span>
        <span class="workspace-path" title={workspace.path}>{workspace.path}</span>
      </span>
      <span class="header-meta">
        <span class="count">{workspace.visibleSessions.length}</span>
        <span class="time">{workspace.updatedAtLabel}</span>
      </span>
    </button>
  </div>

  {#if !collapsed}
    <div
      class="session-list"
      aria-label={t("workspace.sessionsAria", { name: workspace.name })}
    >
      {#each workspace.visibleSessions as session (session.id)}
        <SessionRow
          {session}
          active={selectionKind === "session" && selectedSessionId === session.id}
          checked={selectedSessionIds.includes(session.id)}
          onSelect={onSelectSession}
          onToggle={onToggleSession}
        />
      {/each}
    </div>
  {/if}
</article>

<style>
  .workspace-card {
    display: grid;
    gap: 4px;
    padding: 6px 4px 8px;
    border-radius: var(--radius-md);
    background: transparent;
  }

  .workspace-card.active .workspace-header {
    background: var(--selection-inactive);
  }

  .workspace-row {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    gap: 2px;
  }

  .disclosure {
    display: grid;
    place-items: center;
    width: 18px;
    height: 22px;
    padding: 0;
    border-radius: 4px;
    color: var(--fg-tertiary);
    background: transparent;
    cursor: pointer;
  }

  .disclosure:hover {
    color: var(--fg-secondary);
    background: var(--selection-inactive);
  }

  .workspace-header {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    background: transparent;
    text-align: left;
    color: inherit;
    transition: background-color 60ms ease;
  }

  .workspace-header:hover {
    background: var(--selection-inactive);
  }

  .header-icon {
    display: grid;
    place-items: center;
    color: var(--fg-tertiary);
  }

  .header-text {
    display: grid;
    gap: 1px;
    min-width: 0;
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .workspace-name {
    overflow: hidden;
    color: var(--fg);
    font-size: 13px;
    font-weight: 600;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .workspace-path {
    overflow: hidden;
    color: var(--fg-tertiary);
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 16px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .header-meta {
    display: grid;
    justify-items: end;
    gap: 1px;
    text-align: right;
  }

  .header-meta .count {
    color: var(--fg-secondary);
    font-size: 11px;
    font-weight: 600;
    line-height: 16px;
    font-variant-numeric: tabular-nums;
  }

  .header-meta .time {
    color: var(--fg-tertiary);
    font-size: 11px;
    line-height: 15px;
    font-variant-numeric: tabular-nums;
  }

  .session-list {
    display: grid;
    gap: 1px;
    padding: 0 0 0 22px;
  }
</style>
