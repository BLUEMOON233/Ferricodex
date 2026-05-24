<script lang="ts">
  import EmptyDetail from "./EmptyDetail.svelte";
  import SessionDetail from "./SessionDetail.svelte";
  import SettingsView from "./SettingsView.svelte";
  import WorkspaceDetail from "./WorkspaceDetail.svelte";
  import { StatePanel } from "$lib/components/ui";
  import type {
    CodexHomeStatus,
    CodexTranscript,
    CodexTranscriptMessage,
    Session,
    WorkspaceMetadata,
  } from "$lib/codex";
  import type { Workspace } from "$lib/workspace";

  type Props = {
    filter: "active" | "archived" | "settings";
    selectionKind: "session" | "workspace";
    selectedSession: Session | null;
    selectedWorkspace: Workspace | null;
    sessionWorkspace: Workspace | null;
    workspaceMetadata: WorkspaceMetadata | null;
    workspaceMetadataError: string;
    workspaceMetadataLoading: boolean;
    transcript: CodexTranscript | null;
    transcriptError: string;
    transcriptLoading: boolean;
    transcriptQuery: string;
    transcriptRoleFilter: string;
    transcriptRoleOptions: string[];
    filteredTranscriptMessages: CodexTranscriptMessage[];
    transcriptFilterActive: boolean;
    archiveBusy: boolean;
    sessionTrashBusy: boolean;
    workspaceTrashBusy: boolean;
    archiveError: string;
    trashError: string;
    codexHome: CodexHomeStatus | null;
    onOpenWorkspaceFolder: () => void;
    onToggleArchive: () => void;
    onTrashSession: () => void;
    onTrashWorkspace: () => void;
    onOpenCodexHome: () => void;
    onSelectSession: (session: Session) => void;
    onTranscriptQueryChange: (value: string) => void;
    onTranscriptRoleChange: (value: string) => void;
    onClearTranscriptFilters: () => void;
  };

  let {
    filter,
    selectionKind,
    selectedSession,
    selectedWorkspace,
    sessionWorkspace,
    workspaceMetadata,
    workspaceMetadataError,
    workspaceMetadataLoading,
    transcript,
    transcriptError,
    transcriptLoading,
    transcriptQuery,
    transcriptRoleFilter,
    transcriptRoleOptions,
    filteredTranscriptMessages,
    transcriptFilterActive,
    archiveBusy,
    sessionTrashBusy,
    workspaceTrashBusy,
    archiveError,
    trashError,
    codexHome,
    onOpenWorkspaceFolder,
    onToggleArchive,
    onTrashSession,
    onTrashWorkspace,
    onOpenCodexHome,
    onSelectSession,
    onTranscriptQueryChange,
    onTranscriptRoleChange,
    onClearTranscriptFilters,
  }: Props = $props();
</script>

<section class="detail-panel" aria-label="Selected item details">
  {#if filter === "settings"}
    <SettingsView {codexHome} {onOpenCodexHome} />
  {:else if selectionKind === "workspace" && selectedWorkspace}
    {#if archiveError}
      <StatePanel variant="error">{archiveError}</StatePanel>
    {/if}
    {#if trashError}
      <StatePanel variant="error">{trashError}</StatePanel>
    {/if}
    <WorkspaceDetail
      workspace={selectedWorkspace}
      metadata={workspaceMetadata}
      metadataError={workspaceMetadataError}
      metadataLoading={workspaceMetadataLoading}
      trashBusy={workspaceTrashBusy}
      onOpenFolder={onOpenWorkspaceFolder}
      onTrash={onTrashWorkspace}
      {onSelectSession}
    />
  {:else if selectedSession}
    {#if archiveError}
      <StatePanel variant="error">{archiveError}</StatePanel>
    {/if}
    {#if trashError}
      <StatePanel variant="error">{trashError}</StatePanel>
    {/if}
    <SessionDetail
      session={selectedSession}
      workspace={sessionWorkspace}
      {transcript}
      {transcriptError}
      transcriptLoading={transcriptLoading}
      {transcriptQuery}
      {transcriptRoleFilter}
      {transcriptRoleOptions}
      filteredMessages={filteredTranscriptMessages}
      filterActive={transcriptFilterActive}
      {archiveBusy}
      trashBusy={sessionTrashBusy}
      onOpenWorkspace={onOpenWorkspaceFolder}
      {onToggleArchive}
      onTrash={onTrashSession}
      {onTranscriptQueryChange}
      {onTranscriptRoleChange}
      {onClearTranscriptFilters}
    />
  {:else}
    <EmptyDetail />
  {/if}
</section>

<style>
  .detail-panel {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    align-content: start;
    gap: 14px;
    min-width: 0;
    min-height: 0;
    padding: 18px 20px 24px;
    overflow-x: hidden;
    overflow-y: auto;
    background: var(--bg-window);
  }
</style>
