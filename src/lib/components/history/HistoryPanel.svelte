<script lang="ts">
  import { FolderOpen, Inbox, Search } from "@lucide/svelte";
  import { IconButton, Kbd, SearchField, Skeleton, StatePanel } from "$lib/components/ui";
  import { modKeyLabel } from "$lib/platform";
  import BulkToolbar from "./BulkToolbar.svelte";
  import GlobalSearchPanel from "./GlobalSearchPanel.svelte";
  import WorkspaceSection from "./WorkspaceSection.svelte";
  import { t } from "$lib/i18n.svelte";
  import type {
    CodexHomeStatus,
    CodexSearchResponse,
    CodexSearchResult,
    CodexSearchScope,
    Session,
  } from "$lib/codex";
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

  type Filter = "active" | "archived";

  type Props = {
    filter: Filter;
    title: string;
    description: string;
    codexHome: CodexHomeStatus | null;
    isLoading: boolean;
    loadError: string;
    openerError: string;
    sections: Section[];
    visibleWorkspaceCount: number;
    visibleSessionCount: number;
    selectionKind: "session" | "workspace";
    selectedSessionId: string;
    selectedWorkspaceId: string;
    selectedSessionIds: string[];
    visibleSessions: Session[];
    bulkBusy: boolean;
    query: string;
    globalSearchQuery: string;
    globalSearchScope: CodexSearchScope;
    globalSearchResponse: CodexSearchResponse | null;
    globalSearchError: string;
    globalSearchLoading: boolean;
    onQueryChange: (value: string) => void;
    onSelectWorkspace: (workspace: Workspace) => void;
    onSelectSession: (session: Session) => void;
    onToggleSession: (session: Session, checked: boolean) => void;
    onSelectVisible: () => void;
    onClearSelection: () => void;
    onBulkTrash: () => void;
    onOpenCodexHome: () => void;
    onGlobalSearchQueryChange: (value: string) => void;
    onGlobalSearchScopeChange: (scope: CodexSearchScope) => void;
    onGlobalSearchSubmit: () => void;
    onGlobalSearchClear: () => void;
    onGlobalSearchSelectResult: (result: CodexSearchResult) => void;
  };

  let {
    filter,
    title,
    description,
    codexHome,
    isLoading,
    loadError,
    openerError,
    sections,
    visibleWorkspaceCount,
    visibleSessionCount,
    selectionKind,
    selectedSessionId,
    selectedWorkspaceId,
    selectedSessionIds,
    visibleSessions,
    bulkBusy,
    query,
    globalSearchQuery,
    globalSearchScope,
    globalSearchResponse,
    globalSearchError,
    globalSearchLoading,
    onQueryChange,
    onSelectWorkspace,
    onSelectSession,
    onToggleSession,
    onSelectVisible,
    onClearSelection,
    onBulkTrash,
    onOpenCodexHome,
    onGlobalSearchQueryChange,
    onGlobalSearchScopeChange,
    onGlobalSearchSubmit,
    onGlobalSearchClear,
    onGlobalSearchSelectResult,
  }: Props = $props();

  let searchEl: SearchField | undefined = $state();

  const allVisibleSelected = $derived(
    visibleSessions.length > 0 &&
      visibleSessions.every((session) => selectedSessionIds.includes(session.id)),
  );

  export function focusSearch() {
    searchEl?.focus();
  }
</script>

<section class="history-panel" aria-label={t("sidebar.brand")}>
  <header class="head">
    <div class="head-text">
      <span class="eyebrow">
        {filter === "archived" ? t("history.eyebrow.archive") : t("history.eyebrow.library")}
      </span>
      <h1>{title}</h1>
      <p class="head-description">{description}</p>
    </div>
    <IconButton
      aria-label={t("history.openCodexHome")}
      title={t("history.openCodexHome")}
      disabled={!codexHome?.exists}
      onclick={onOpenCodexHome}
    >
      <FolderOpen size={15} />
    </IconButton>
  </header>

  <div class="search-bar">
    <SearchField
      bind:this={searchEl}
      value={query}
      placeholder={t("history.searchPlaceholder")}
      oninput={(event) => onQueryChange((event.currentTarget as HTMLInputElement).value)}
    >
      {#snippet trailing()}
        <span class="kbd-hint" aria-hidden="true">
          <Kbd>{modKeyLabel}</Kbd>
          <Kbd>F</Kbd>
        </span>
      {/snippet}
    </SearchField>
  </div>

  <GlobalSearchPanel
    query={globalSearchQuery}
    scope={globalSearchScope}
    response={globalSearchResponse}
    error={globalSearchError}
    loading={globalSearchLoading}
    onQueryChange={onGlobalSearchQueryChange}
    onScopeChange={onGlobalSearchScopeChange}
    onSubmit={onGlobalSearchSubmit}
    onClear={onGlobalSearchClear}
    onSelectResult={onGlobalSearchSelectResult}
  />

  <BulkToolbar
    selectedCount={selectedSessionIds.length}
    visibleCount={visibleSessions.length}
    canSelectVisible={visibleSessions.length > 0 && !allVisibleSelected}
    busy={bulkBusy}
    onSelectVisible={onSelectVisible}
    onClear={onClearSelection}
    onTrash={onBulkTrash}
  />

  <div class="meta-row">
    <span>
      {t("history.workspaces", { count: visibleWorkspaceCount })}
      <span class="dim">·</span>
      {t("history.sessions", { count: visibleSessionCount })}
    </span>
    <span class="dim">{isLoading ? t("common.loading") : t("common.ready")}</span>
  </div>

  {#if openerError}
    <StatePanel variant="error">{openerError}</StatePanel>
  {/if}

  <div class="list" aria-live="polite">
    {#if isLoading && sections.length === 0}
      <div class="skeleton-stack">
        <Skeleton height="44px" radius="var(--radius-sm)" />
        <Skeleton height="44px" radius="var(--radius-sm)" />
        <Skeleton height="44px" radius="var(--radius-sm)" />
        <Skeleton height="44px" radius="var(--radius-sm)" />
        <Skeleton height="44px" radius="var(--radius-sm)" />
      </div>
    {:else if loadError}
      <StatePanel variant="error">{loadError}</StatePanel>
    {:else if sections.length === 0}
      <div class="empty">
        <span class="empty-icon" aria-hidden="true">
          {#if query}
            <Search size={22} strokeWidth={1.6} />
          {:else}
            <Inbox size={22} strokeWidth={1.6} />
          {/if}
        </span>
        <strong>
          {#if query}
            {t("history.empty.searchTitle")}
          {:else if filter === "archived"}
            {t("history.empty.archivedTitle")}
          {:else}
            {t("history.empty.activeTitle")}
          {/if}
        </strong>
        <span>
          {#if query}
            {t("history.empty.searchHint")}
          {:else if filter === "archived"}
            {t("history.empty.archivedHint")}
          {:else}
            {t("history.empty.activeHint")}
          {/if}
        </span>
      </div>
    {:else}
      {#each sections as section (section.id)}
        <WorkspaceSection
          {section}
          {selectionKind}
          {selectedSessionId}
          {selectedWorkspaceId}
          {selectedSessionIds}
          forceExpanded={query.trim() !== ""}
          {onSelectWorkspace}
          {onSelectSession}
          {onToggleSession}
        />
      {/each}
    {/if}
  </div>
</section>

<style>
  .history-panel {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
    padding: 14px 12px;
    background: var(--bg-content);
    border-right: 1px solid var(--separator);
    overflow-y: auto;
  }

  .head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
    padding: 0 4px 4px;
  }

  .head-text {
    min-width: 0;
  }

  .eyebrow {
    color: var(--fg-tertiary);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .head h1 {
    margin-top: 2px;
    color: var(--fg);
    font-size: 18px;
    font-weight: 600;
    line-height: 1.25;
  }

  .head-description {
    margin-top: 3px;
    color: var(--fg-tertiary);
    font-size: 12px;
    line-height: 1.45;
  }

  .search-bar {
    display: grid;
    padding: 0 4px;
  }

  .kbd-hint {
    display: inline-flex;
    gap: 2px;
  }

  .meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 6px 0;
    color: var(--fg-secondary);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }

  .dim {
    color: var(--fg-tertiary);
  }

  .list {
    display: grid;
    gap: 4px;
    padding-bottom: 8px;
  }

  .skeleton-stack {
    display: grid;
    gap: 6px;
    padding: 8px 6px;
  }

  .empty {
    display: grid;
    place-items: center;
    gap: 6px;
    padding: 40px 16px;
    text-align: center;
    color: var(--fg-tertiary);
  }

  .empty-icon {
    display: grid;
    width: 44px;
    height: 44px;
    place-items: center;
    border-radius: 999px;
    background: var(--bg-input-soft);
    color: var(--fg-tertiary);
  }

  .empty strong {
    color: var(--fg-secondary);
    font-size: 13px;
    font-weight: 600;
  }

  .empty span {
    max-width: 280px;
    font-size: 12px;
    line-height: 1.45;
  }
</style>
