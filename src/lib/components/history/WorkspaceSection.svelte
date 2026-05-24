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
    gap: 4px;
  }

  .section-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 8px 8px 4px;
  }

  .section-toggle {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 0;
    color: var(--fg-tertiary);
    background: transparent;
    cursor: pointer;
  }

  .section-toggle:hover {
    color: var(--fg-secondary);
  }

  .section-toggle :global(svg) {
    color: var(--fg-tertiary);
    transition: transform 80ms ease;
  }

  .section-title {
    color: var(--fg-tertiary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .section-meta {
    color: var(--fg-tertiary);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }

  .workspace-stack {
    display: grid;
    gap: 2px;
  }
</style>
