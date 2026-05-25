<script lang="ts">
  import { ChevronDown, ChevronRight } from "@lucide/svelte";
  import WorkspaceCard from "./WorkspaceCard.svelte";
  import { formatCount } from "$lib/formatting";
  import { collapse } from "$lib/collapse.svelte";
  import { t } from "$lib/i18n.svelte";
  import type { Session } from "$lib/codex";
  import type { Workspace } from "$lib/workspace";

  type WorkspaceListItem = Workspace & {
    visibleSessions: Session[];
    visibleActiveCount: number;
    visibleArchivedCount: number;
  };

  type Section = {
    id: string;
    title: string;
    subtitle: string;
    workspaces: WorkspaceListItem[];
    sessionCount: number;
  };

  type Props = {
    section: Section;
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
    section,
    selectedSessionId,
    selectedWorkspaceId,
    selectionKind,
    selectedSessionIds,
    forceExpanded = false,
    onSelectWorkspace,
    onSelectSession,
    onToggleSession,
  }: Props = $props();

  const collapsed = $derived(!forceExpanded && collapse.isSectionCollapsed(section.id));
</script>

<section class="workspace-section">
  <header class="section-heading">
    <button
      type="button"
      class="section-toggle"
      aria-expanded={!collapsed}
      aria-label={collapsed
        ? t("section.toggle.expand", { title: section.title })
        : t("section.toggle.collapse", { title: section.title })}
      onclick={() => collapse.toggleSection(section.id)}
    >
      {#if collapsed}
        <ChevronRight size={12} />
      {:else}
        <ChevronDown size={12} />
      {/if}
      <span class="section-title">{section.title}</span>
    </button>
    <span class="section-meta">{formatCount(section.sessionCount)}</span>
  </header>

  {#if !collapsed}
    <div class="workspace-stack">
      {#each section.workspaces as workspace (workspace.id)}
        <WorkspaceCard
          {workspace}
          {selectedSessionId}
          {selectedWorkspaceId}
          {selectionKind}
          {selectedSessionIds}
          {forceExpanded}
          {onSelectWorkspace}
          {onSelectSession}
          {onToggleSession}
        />
      {/each}
    </div>
  {/if}
</section>

<style>
  .workspace-section {
    display: grid;
    gap: 6px;
  }

  .section-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin: 8px 4px 2px;
    padding: 7px 8px;
    border: 1px solid var(--separator);
    border-radius: var(--radius-sm);
    background: var(--bg-input-soft);
  }

  .section-toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    padding: 0;
    color: var(--fg-secondary);
    background: transparent;
    cursor: pointer;
  }

  .section-toggle:hover {
    color: var(--fg);
  }

  .section-toggle :global(svg) {
    flex: 0 0 auto;
    color: var(--fg-secondary);
    transition: transform 80ms ease;
  }

  .section-title {
    overflow: hidden;
    color: var(--fg-secondary);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .section-meta {
    flex: 0 0 auto;
    padding: 1px 6px;
    border-radius: 999px;
    color: var(--fg-secondary);
    background: var(--bg-surface);
    font-size: 11px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .workspace-stack {
    display: grid;
    gap: 2px;
  }
</style>
