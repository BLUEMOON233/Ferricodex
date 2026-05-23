<script lang="ts">
  import {
    Archive,
    Database,
    FolderOpen,
    History,
    Search,
    Settings,
    ShieldCheck,
    Trash2,
  } from "@lucide/svelte";
  import {
    getCodexHomeStatus,
    getCodexTranscript,
    getWorkspaceMetadata,
    listCodexThreads,
    moveGeneratedWorkspaceSessionToTrash,
    moveThreadToTrash,
    moveThreadsToTrash,
    setThreadArchiveState,
    toSession,
    type CodexHomeStatus,
    type CodexTranscript,
    type Session,
    type WorkspaceMetadata,
  } from "$lib/codex";
  import { formatCount, formatDate, formatSize } from "$lib/formatting";
  import {
    groupSessionsByWorkspace,
    normalizeWorkspacePath,
    workspaceSourceDescription,
    type Workspace,
  } from "$lib/workspace";
  import { openLocalPath } from "$lib/opener";

  type HistoryFilter = "active" | "archived" | "settings";
  type SelectionKind = "session" | "workspace";
  type WorkspaceListItem = Workspace & {
    visibleSessions: Session[];
    visibleActiveCount: number;
    visibleArchivedCount: number;
  };
  type WorkspaceSection = {
    id: "projects" | "conversations";
    title: string;
    subtitle: string;
    workspaces: WorkspaceListItem[];
    sessionCount: number;
  };

  let query = $state("");
  let historyFilter = $state<HistoryFilter>("active");
  let selectedKind = $state<SelectionKind>("session");
  let selectedId = $state("");
  let sessions = $state<Session[]>([]);
  let codexHome = $state<CodexHomeStatus | null>(null);
  let workspaceMetadataByPath = $state<Record<string, WorkspaceMetadata>>({});
  let workspaceMetadataErrorsByPath = $state<Record<string, string>>({});
  let transcriptByPath = $state<Record<string, CodexTranscript>>({});
  let transcriptErrorsByPath = $state<Record<string, string>>({});
  let loadingWorkspacePath = $state("");
  let loadingTranscriptPath = $state("");
  let isLoading = $state(true);
  let loadError = $state("");
  let openerError = $state("");
  let archiveError = $state("");
  let archiveActionSessionId = $state("");
  let pendingArchiveSessionId = $state("");
  let pendingArchiveNextArchived = $state<boolean | null>(null);
  let trashError = $state("");
  let trashActionSessionId = $state("");
  let pendingTrashSessionId = $state("");
  let pendingTrashWorkspaceId = $state("");
  let transcriptQuery = $state("");
  let transcriptRoleFilter = $state("all");
  let lastTranscriptPath = "";

  function errorMessage(error: unknown) {
    return error instanceof Error ? error.message : String(error);
  }

  function resetTranscriptFilters() {
    transcriptQuery = "";
    transcriptRoleFilter = "all";
  }

  function selectHistoryFilter(filter: "active" | "archived") {
    historyFilter = filter;
    selectedKind = "session";
    selectedId = "";
    openerError = "";
  }

  function selectSettings() {
    historyFilter = "settings";
    selectedKind = "session";
    selectedId = "";
    openerError = "";
  }

  function sessionMatchesFilter(session: Session) {
    if (historyFilter === "settings") {
      return false;
    }

    return historyFilter === "archived" ? session.archived : !session.archived;
  }

  function sessionMatchesSearch(session: Session, normalizedQuery: string) {
    if (!normalizedQuery) {
      return true;
    }

    const value = `${session.title} ${session.cwd} ${session.preview} ${session.id} ${session.model ?? ""}`;
    return value.toLowerCase().includes(normalizedQuery);
  }

  function workspaceMatchesSearch(workspace: Workspace, normalizedQuery: string) {
    if (!normalizedQuery) {
      return true;
    }

    const value = `${workspace.name} ${workspace.path} ${workspace.sourceLabel}`;
    return value.toLowerCase().includes(normalizedQuery);
  }

  function visibleWorkspace(workspace: Workspace, normalizedQuery: string): WorkspaceListItem | null {
    const workspaceMatches = workspaceMatchesSearch(workspace, normalizedQuery);
    const visibleSessions = workspace.sessions
      .filter(sessionMatchesFilter)
      .filter((session) => workspaceMatches || sessionMatchesSearch(session, normalizedQuery))
      .sort((left, right) => right.updatedAt - left.updatedAt || left.title.localeCompare(right.title));

    if (visibleSessions.length === 0) {
      return null;
    }

    return {
      ...workspace,
      visibleSessions,
      visibleActiveCount: visibleSessions.filter((session) => !session.archived).length,
      visibleArchivedCount: visibleSessions.filter((session) => session.archived).length,
    };
  }

  function workspaceTrashActionKey(workspace: Workspace) {
    return `workspace:${workspace.id}`;
  }

  function selectSession(session: Session) {
    selectedKind = "session";
    selectedId = session.id;
    openerError = "";
  }

  function selectWorkspace(workspace: Workspace) {
    selectedKind = "workspace";
    selectedId = workspace.id;
    openerError = "";
  }

  function clearPendingArchiveAction() {
    pendingArchiveSessionId = "";
    pendingArchiveNextArchived = null;
  }

  function clearPendingTrashAction() {
    pendingTrashSessionId = "";
    pendingTrashWorkspaceId = "";
  }

  async function openPathWithFeedback(path: string | null | undefined, label: string) {
    const trimmedPath = path?.trim() ?? "";

    if (!trimmedPath) {
      openerError = `${label} path is unavailable.`;
      return;
    }

    try {
      openerError = "";
      await openLocalPath(trimmedPath);
    } catch (error) {
      openerError = `Could not open ${label}: ${errorMessage(error)}`;
    }
  }

  async function openCodexHomeDirectory() {
    if (!codexHome?.exists) {
      openerError = codexHome?.path
        ? "Codex home directory does not exist on disk."
        : "Codex home directory is unavailable.";
      return;
    }

    await openPathWithFeedback(codexHome.path, "Codex home directory");
  }

  async function openSelectedWorkspaceFolder() {
    if (!selectedWorkspace) {
      openerError = "No workspace is selected.";
      return;
    }

    if (selectedWorkspaceMetadata && !selectedWorkspaceMetadata.exists) {
      openerError = "Selected workspace does not exist on disk.";
      return;
    }

    await openPathWithFeedback(selectedWorkspace.path, "workspace folder");
  }

  function requestSelectedSessionArchiveState() {
    if (!selectedSession) {
      archiveError = "No session is selected.";
      return;
    }

    clearPendingTrashAction();
    pendingArchiveSessionId = selectedSession.id;
    pendingArchiveNextArchived = !selectedSession.archived;
    archiveError = "";
    trashError = "";
  }

  async function confirmSelectedSessionArchiveState() {
    if (!pendingArchiveSessionId || pendingArchiveNextArchived === null) {
      archiveError = "No archive action is pending.";
      return;
    }

    const sessionId = pendingArchiveSessionId;
    const nextArchived = pendingArchiveNextArchived;
    const action = nextArchived ? "archive" : "restore";

    archiveActionSessionId = sessionId;
    archiveError = "";

    try {
      await setThreadArchiveState(sessionId, nextArchived);
      await loadSessions();

      historyFilter = nextArchived ? "archived" : "active";
      selectedKind = "session";
      selectedId = sessionId;
    } catch (error) {
      archiveError = `Could not ${action} session: ${errorMessage(error)}`;
    } finally {
      archiveActionSessionId = "";
      clearPendingArchiveAction();
    }
  }

  function requestSelectedSessionTrash() {
    if (!selectedSession) {
      trashError = "No session is selected.";
      return;
    }

    clearPendingArchiveAction();
    pendingTrashSessionId = selectedSession.id;
    pendingTrashWorkspaceId = "";
    archiveError = "";
    trashError = "";
  }

  function requestSelectedWorkspaceTrash() {
    if (!selectedWorkspace) {
      trashError = "No workspace is selected.";
      return;
    }

    clearPendingArchiveAction();
    pendingTrashSessionId = "";
    pendingTrashWorkspaceId = selectedWorkspace.id;
    archiveError = "";
    trashError = "";
  }

  async function confirmSelectedSessionTrash() {
    if (!pendingTrashSessionId) {
      trashError = "No trash action is pending.";
      return;
    }

    const sessionId = pendingTrashSessionId;

    trashActionSessionId = sessionId;
    trashError = "";

    try {
      await moveThreadToTrash(sessionId);
      await loadSessions();
    } catch (error) {
      trashError = `Could not move session to Trash: ${errorMessage(error)}`;
    } finally {
      trashActionSessionId = "";
      clearPendingTrashAction();
    }
  }

  async function confirmGeneratedWorkspaceSessionTrash(
    sessionId: string,
    saveWorkspaceCopy: boolean,
  ) {
    const trimmedSessionId = sessionId.trim();

    if (!trimmedSessionId) {
      trashError = "No generated workspace session is selected.";
      return;
    }

    trashActionSessionId = trimmedSessionId;
    trashError = "";

    try {
      await moveGeneratedWorkspaceSessionToTrash(trimmedSessionId, saveWorkspaceCopy);
      await loadSessions();
    } catch (error) {
      trashError = `Could not delete generated workspace session: ${errorMessage(error)}`;
    } finally {
      trashActionSessionId = "";
      clearPendingTrashAction();
    }
  }

  async function confirmWorkspaceHistoryTrash() {
    if (!pendingTrashWorkspace) {
      trashError = "No workspace trash action is pending.";
      return;
    }

    const workspace = pendingTrashWorkspace;
    const sessionIds = workspace.sessions.map((session) => session.id);

    if (sessionIds.length === 0) {
      trashError = "Selected workspace has no sessions to remove.";
      return;
    }

    const actionKey = workspaceTrashActionKey(workspace);
    trashActionSessionId = actionKey;
    trashError = "";

    try {
      await moveThreadsToTrash(sessionIds);
      await loadSessions();
    } catch (error) {
      trashError = `Could not remove workspace from Codex history: ${errorMessage(error)}`;
    } finally {
      trashActionSessionId = "";
      clearPendingTrashAction();
    }
  }

  async function confirmPendingSessionWorkspaceHistoryTrash() {
    if (!pendingTrashSessionWorkspace) {
      trashError = "Could not find the selected session workspace.";
      return;
    }

    const workspace = pendingTrashSessionWorkspace;
    const sessionIds = workspace.sessions.map((session) => session.id);

    if (sessionIds.length === 0) {
      trashError = "Selected workspace has no sessions to remove.";
      return;
    }

    const actionKey = workspaceTrashActionKey(workspace);
    trashActionSessionId = actionKey;
    trashError = "";

    try {
      await moveThreadsToTrash(sessionIds);
      await loadSessions();
    } catch (error) {
      trashError = `Could not remove workspace from Codex history: ${errorMessage(error)}`;
    } finally {
      trashActionSessionId = "";
      clearPendingTrashAction();
    }
  }

  async function loadTranscriptForSession(session: Session, force = false) {
    const path = session.rolloutPath.trim();

    if (!path) {
      transcriptErrorsByPath = {
        ...transcriptErrorsByPath,
        [session.id]: "Transcript path is unavailable.",
      };
      return;
    }

    if (loadingTranscriptPath === path || (!force && transcriptByPath[path])) {
      return;
    }

    loadingTranscriptPath = path;
    const { [path]: _clearedError, ...remainingErrors } = transcriptErrorsByPath;
    transcriptErrorsByPath = remainingErrors;

    try {
      const transcript = await getCodexTranscript(path);
      transcriptByPath = {
        ...transcriptByPath,
        [path]: transcript,
      };
    } catch (error) {
      transcriptErrorsByPath = {
        ...transcriptErrorsByPath,
        [path]: errorMessage(error),
      };
    } finally {
      if (loadingTranscriptPath === path) {
        loadingTranscriptPath = "";
      }
    }
  }

  async function loadSessions() {
    isLoading = true;
    loadError = "";

    try {
      codexHome = await getCodexHomeStatus();
      const threads = await listCodexThreads();
      sessions = threads.map(toSession);
      selectedKind = "session";
      selectedId = sessions[0]?.id ?? "";
      workspaceMetadataByPath = {};
      workspaceMetadataErrorsByPath = {};
      transcriptByPath = {};
      transcriptErrorsByPath = {};
      loadingWorkspacePath = "";
      loadingTranscriptPath = "";
      openerError = "";
      archiveError = "";
      archiveActionSessionId = "";
      clearPendingArchiveAction();
      trashError = "";
      trashActionSessionId = "";
      clearPendingTrashAction();
      transcriptQuery = "";
      transcriptRoleFilter = "all";
    } catch (error) {
      loadError = errorMessage(error);
      sessions = [];
      selectedKind = "session";
      selectedId = "";
      workspaceMetadataByPath = {};
      workspaceMetadataErrorsByPath = {};
      transcriptByPath = {};
      transcriptErrorsByPath = {};
      loadingWorkspacePath = "";
      loadingTranscriptPath = "";
      openerError = "";
      archiveError = "";
      archiveActionSessionId = "";
      clearPendingArchiveAction();
      trashError = "";
      trashActionSessionId = "";
      clearPendingTrashAction();
      transcriptQuery = "";
      transcriptRoleFilter = "all";
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    loadSessions();
  });

  const workspaces = $derived.by((): Workspace[] =>
    groupSessionsByWorkspace(sessions, codexHome?.path),
  );

  const activeSessionCount = $derived(sessions.filter((session) => !session.archived).length);
  const archivedSessionCount = $derived(sessions.filter((session) => session.archived).length);

  const currentHistoryTitle = $derived(
    historyFilter === "settings" ? "Settings" : historyFilter === "archived" ? "Archived" : "Active",
  );

  const currentHistoryDescription = $derived(
    historyFilter === "settings"
      ? "App preferences and local Codex store status."
      : historyFilter === "archived"
        ? "Archived sessions grouped under their original workspace."
        : "Active sessions grouped under their workspace.",
  );

  const workspaceSections = $derived.by((): WorkspaceSection[] => {
    const normalizedQuery = query.trim().toLowerCase();
    const projectWorkspaces: WorkspaceListItem[] = [];
    const conversationWorkspaces: WorkspaceListItem[] = [];

    for (const workspace of workspaces) {
      const item = visibleWorkspace(workspace, normalizedQuery);

      if (!item) {
        continue;
      }

      if (item.source === "codexTaskFolder") {
        conversationWorkspaces.push(item);
      } else {
        projectWorkspaces.push(item);
      }
    }

    const sections: WorkspaceSection[] = [
      {
        id: "projects",
        title: "Projects",
        subtitle: "Real project folders and Codex worktrees, with sessions stacked below.",
        workspaces: projectWorkspaces,
        sessionCount: projectWorkspaces.reduce(
          (total, workspace) => total + workspace.visibleSessions.length,
          0,
        ),
      },
      {
        id: "conversations",
        title: "Conversations",
        subtitle: "Codex-generated task folders under Documents/Codex, shown as conversations.",
        workspaces: conversationWorkspaces,
        sessionCount: conversationWorkspaces.reduce(
          (total, workspace) => total + workspace.visibleSessions.length,
          0,
        ),
      },
    ];

    return sections.filter((section) => section.workspaces.length > 0);
  });

  const visibleSessionCount = $derived(
    workspaceSections.reduce((total, section) => total + section.sessionCount, 0),
  );

  const visibleWorkspaceCount = $derived(
    workspaceSections.reduce((total, section) => total + section.workspaces.length, 0),
  );

  const firstVisibleSession = $derived.by((): Session | null => {
    for (const section of workspaceSections) {
      for (const workspace of section.workspaces) {
        const session = workspace.visibleSessions[0];

        if (session) {
          return session;
        }
      }
    }

    return null;
  });

  const firstVisibleWorkspace = $derived.by((): Workspace | null => {
    for (const section of workspaceSections) {
      const workspace = section.workspaces[0];

      if (workspace) {
        return workspaces.find((candidate) => candidate.id === workspace.id) ?? workspace;
      }
    }

    return null;
  });

  const selectedSession = $derived.by((): Session | null => {
    if (historyFilter === "settings" || selectedKind !== "session") {
      return null;
    }

    const selected = sessions.find((session) => session.id === selectedId);

    if (selected && sessionMatchesFilter(selected)) {
      return selected;
    }

    return firstVisibleSession;
  });

  const selectedSessionWorkspace = $derived(
    selectedSession
      ? (workspaces.find(
          (workspace) => workspace.id === normalizeWorkspacePath(selectedSession.cwd),
        ) ?? null)
      : null,
  );

  const selectedWorkspace = $derived.by((): Workspace | null => {
    if (historyFilter === "settings") {
      return null;
    }

    if (selectedKind === "workspace") {
      return workspaces.find((workspace) => workspace.id === selectedId) ?? firstVisibleWorkspace;
    }

    return selectedSessionWorkspace;
  });

  const pendingArchiveSession = $derived(
    pendingArchiveSessionId
      ? (sessions.find((session) => session.id === pendingArchiveSessionId) ?? null)
      : null,
  );

  const pendingTrashSession = $derived(
    pendingTrashSessionId
      ? (sessions.find((session) => session.id === pendingTrashSessionId) ?? null)
      : null,
  );

  const pendingTrashWorkspace = $derived(
    pendingTrashWorkspaceId
      ? (workspaces.find((workspace) => workspace.id === pendingTrashWorkspaceId) ?? null)
      : null,
  );

  const pendingTrashSessionWorkspace = $derived(
    pendingTrashSession
      ? (workspaces.find(
          (workspace) => workspace.id === normalizeWorkspacePath(pendingTrashSession.cwd),
        ) ?? null)
      : null,
  );

  const pendingTrashSessionIsGeneratedWorkspace = $derived(
    pendingTrashSessionWorkspace?.source === "codexTaskFolder",
  );

  const pendingTrashWorkspacePrimarySession = $derived(pendingTrashWorkspace?.sessions[0] ?? null);

  const selectedWorkspaceMetadata = $derived(
    selectedWorkspace ? (workspaceMetadataByPath[selectedWorkspace.id] ?? null) : null,
  );

  const selectedWorkspaceMetadataError = $derived(
    selectedWorkspace ? (workspaceMetadataErrorsByPath[selectedWorkspace.id] ?? "") : "",
  );

  const selectedTranscriptPath = $derived(selectedSession?.rolloutPath.trim() ?? "");

  const selectedTranscript = $derived(
    selectedTranscriptPath ? (transcriptByPath[selectedTranscriptPath] ?? null) : null,
  );

  const selectedTranscriptError = $derived(
    selectedTranscriptPath
      ? (transcriptErrorsByPath[selectedTranscriptPath] ?? "")
      : selectedSession
        ? "Transcript path is unavailable."
        : "",
  );

  const selectedTranscriptIsLoading = $derived(
    selectedTranscriptPath !== "" && loadingTranscriptPath === selectedTranscriptPath,
  );

  const transcriptRoleOptions = $derived.by(() => {
    const roles = selectedTranscript?.messages.map((message) => message.role).filter(Boolean) ?? [];
    return [...new Set(roles)].sort((left, right) => left.localeCompare(right));
  });

  const filteredTranscriptMessages = $derived.by(() => {
    const messages = selectedTranscript?.messages ?? [];
    const normalizedQuery = transcriptQuery.trim().toLowerCase();

    return messages.filter((message) => {
      const matchesRole = transcriptRoleFilter === "all" || message.role === transcriptRoleFilter;
      const searchableValue = `${message.role} ${message.lineNumber} ${message.timestamp ?? ""} ${message.text}`.toLowerCase();
      const matchesQuery = !normalizedQuery || searchableValue.includes(normalizedQuery);

      return matchesRole && matchesQuery;
    });
  });

  const transcriptFilterActive = $derived(
    transcriptQuery.trim() !== "" || transcriptRoleFilter !== "all",
  );

  $effect(() => {
    const path = selectedTranscriptPath;

    if (path === lastTranscriptPath) {
      return;
    }

    lastTranscriptPath = path;
    resetTranscriptFilters();
  });

  $effect(() => {
    if (!selectedWorkspace) {
      return;
    }

    const workspaceId = selectedWorkspace.id;

    if (workspaceMetadataByPath[workspaceId] || loadingWorkspacePath === workspaceId) {
      return;
    }

    loadingWorkspacePath = workspaceId;
    const { [workspaceId]: _clearedError, ...remainingErrors } = workspaceMetadataErrorsByPath;
    workspaceMetadataErrorsByPath = remainingErrors;

    getWorkspaceMetadata(selectedWorkspace.path)
      .then((metadata) => {
        workspaceMetadataByPath = {
          ...workspaceMetadataByPath,
          [workspaceId]: metadata,
        };
      })
      .catch((error) => {
        workspaceMetadataErrorsByPath = {
          ...workspaceMetadataErrorsByPath,
          [workspaceId]: errorMessage(error),
        };
      })
      .finally(() => {
        if (loadingWorkspacePath === workspaceId) {
          loadingWorkspacePath = "";
        }
      });
  });

  $effect(() => {
    if (!selectedSession) {
      return;
    }

    loadTranscriptForSession(selectedSession);
  });
</script>

<svelte:head>
  <title>Codex History Manager</title>
</svelte:head>

<main class="app-shell">
  <aside class="sidebar" aria-label="Navigation">
    <div class="brand">
      <div class="brand-mark">
        <History size={20} strokeWidth={2.2} />
      </div>
      <div>
        <strong>Codex History</strong>
        <span>Local session manager</span>
      </div>
    </div>

    <nav class="nav-list" aria-label="Primary">
      <button
        class:active={historyFilter === "active"}
        class="nav-item"
        type="button"
        onclick={() => selectHistoryFilter("active")}
      >
        <Database size={17} />
        <span>Active</span>
        <small>{formatCount(activeSessionCount)}</small>
      </button>
      <button
        class:active={historyFilter === "archived"}
        class="nav-item"
        type="button"
        onclick={() => selectHistoryFilter("archived")}
      >
        <Archive size={17} />
        <span>Archived</span>
        <small>{formatCount(archivedSessionCount)}</small>
      </button>
      <button
        class:active={historyFilter === "settings"}
        class="nav-item"
        type="button"
        onclick={selectSettings}
      >
        <Settings size={17} />
        <span>Settings</span>
      </button>
    </nav>

    <section class="library-card" aria-label="Unified library">
      <p class="eyebrow">Unified Library</p>
      <p>
        The list stays workspace-first: projects contain stacked sessions, and Codex-generated
        folders are grouped as conversations.
      </p>
    </section>

    <section class="storage-note" aria-label="Storage policy">
      <ShieldCheck size={18} />
      <div>
        <strong>Codex-safe design</strong>
        <span>Writes are explicit, confirmed, and scoped to Codex-compatible actions.</span>
      </div>
    </section>
  </aside>

  <section class="history-panel" aria-label="Codex history">
    <header class="panel-header">
      <div>
        <p class="eyebrow">Local Codex store</p>
        <h1>{currentHistoryTitle}</h1>
        <p class="panel-description">{currentHistoryDescription}</p>
        {#if codexHome?.path}
          <p class="home-path" title={codexHome.path}>{codexHome.path}</p>
        {/if}
      </div>
      <button
        class="icon-button"
        type="button"
        aria-label="Open Codex data directory"
        title="Open Codex data directory"
        onclick={openCodexHomeDirectory}
      >
        <FolderOpen size={18} />
      </button>
    </header>

    {#if historyFilter !== "settings"}
      <label class="search-box">
        <Search size={18} />
        <input bind:value={query} type="search" placeholder="Search project, path, title, preview, or id" />
      </label>
    {/if}

    <div class="list-meta">
      {#if historyFilter === "settings"}
        <span>{formatCount(sessions.length)} total sessions / {formatCount(workspaces.length)} workspaces</span>
      {:else}
        <span>{formatCount(visibleWorkspaceCount)} workspaces / {formatCount(visibleSessionCount)} sessions</span>
      {/if}
      <span>{isLoading ? "Loading" : "Ready"}</span>
    </div>

    {#if openerError}
      <div class="state-panel error action-error">{openerError}</div>
    {/if}

    <div class="workspace-list" aria-live="polite">
      {#if isLoading}
        <div class="state-panel">Reading local Codex history...</div>
      {:else if loadError}
        <div class="state-panel error">{loadError}</div>
      {:else if historyFilter === "settings"}
        <section class="settings-panel" aria-label="Settings">
          <h2>Settings</h2>
          <p>
            Settings are currently read-only. Use this page to inspect the local Codex data location
            and confirm the app is operating on the expected store.
          </p>
          <dl class="settings-list">
            <div>
              <dt>Codex Home</dt>
              <dd title={codexHome?.path ?? "Unknown"}>{codexHome?.path ?? "Unknown"}</dd>
            </div>
            <div>
              <dt>Source</dt>
              <dd>{codexHome?.source ?? "Unknown"}</dd>
            </div>
            <div>
              <dt>State DB</dt>
              <dd>{codexHome?.stateDbExists ? "Available" : "Missing"}</dd>
            </div>
          </dl>
        </section>
      {:else if workspaceSections.length === 0}
        <div class="state-panel">No sessions match the current search and status filter.</div>
      {:else}
        {#each workspaceSections as section (section.id)}
          <section class="workspace-section" aria-label={section.title}>
            <div class="workspace-section-heading">
              <div>
                <h2>{section.title}</h2>
                <p>{section.subtitle}</p>
              </div>
              <span>{formatCount(section.sessionCount)}</span>
            </div>

            <div class="workspace-stack">
              {#each section.workspaces as workspace (workspace.id)}
                <article
                  class:active={selectedKind === "workspace" && selectedWorkspace?.id === workspace.id}
                  class="workspace-card"
                >
                  <button
                    class="workspace-card-header"
                    type="button"
                    onclick={() => selectWorkspace(workspace)}
                  >
                    <span class="workspace-title-line">
                      <span class="workspace-name" title={workspace.name}>{workspace.name}</span>
                      <span class="workspace-badge">{workspace.sourceLabel}</span>
                    </span>
                    <span class="workspace-path" title={workspace.path}>{workspace.path}</span>
                    <span class="row-footer">
                      <span>
                        {formatCount(workspace.visibleSessions.length)} shown / {formatCount(
                          workspace.sessionCount,
                        )} total
                      </span>
                      <span>Last used {workspace.updatedAtLabel}</span>
                    </span>
                  </button>

                  <div class="nested-sessions" aria-label={`Sessions for ${workspace.name}`}>
                    {#each workspace.visibleSessions as session (session.id)}
                      <button
                        class:active={selectedKind === "session" && selectedSession?.id === session.id}
                        class="nested-session-row"
                        type="button"
                        onclick={() => selectSession(session)}
                      >
                        <span class="session-row-main">
                          <span class="session-dot" class:archived={session.archived}></span>
                          <span class="session-title" title={session.title}>{session.title}</span>
                        </span>
                        <span class="session-preview">{session.preview}</span>
                        <span class="row-footer">
                          <span>{session.updatedAtLabel}</span>
                          <span>{session.archived ? "Archived" : (session.model ?? "Active")}</span>
                        </span>
                      </button>
                    {/each}
                  </div>
                </article>
              {/each}
            </div>
          </section>
        {/each}
      {/if}
    </div>
  </section>

  <section class="detail-panel" aria-label="Selected item details">
    {#if selectedKind === "workspace" && selectedWorkspace}
      <header class="detail-header">
        <div class="detail-heading">
          <p class="eyebrow">Selected workspace</p>
          <h2 title={selectedWorkspace.name}>{selectedWorkspace.name}</h2>
          <p title={selectedWorkspace.path}>{selectedWorkspace.path}</p>
        </div>
        <div class="detail-actions" aria-label="Workspace actions">
          <button
            class="icon-button"
            type="button"
            aria-label="Open workspace folder"
            title="Open workspace folder"
            onclick={openSelectedWorkspaceFolder}
          >
            <FolderOpen size={17} />
          </button>
          <button
            class="icon-button danger"
            type="button"
            aria-label={selectedWorkspace.source === "codexTaskFolder"
              ? "Delete generated workspace"
              : "Remove workspace from Codex history"}
            title={selectedWorkspace.source === "codexTaskFolder"
              ? "Delete generated workspace"
              : "Remove workspace from Codex history"}
            disabled={trashActionSessionId === workspaceTrashActionKey(selectedWorkspace) ||
              trashActionSessionId === selectedWorkspace.sessions[0]?.id}
            onclick={requestSelectedWorkspaceTrash}
          >
            <Trash2 size={17} />
          </button>
        </div>
      </header>

      {#if trashError}
        <div class="state-panel error action-error">{trashError}</div>
      {/if}

      {#if pendingTrashWorkspace && pendingTrashWorkspace.id === selectedWorkspace.id}
        <section class="confirmation-card" aria-label="Confirm workspace trash action">
          <div>
            {#if pendingTrashWorkspace.source === "codexTaskFolder" && pendingTrashWorkspacePrimarySession}
              <h3>Delete this generated conversation?</h3>
              <p>
                This Codex-generated workspace is treated as bound to its session. Deletion removes
                the session from Codex history, moves its transcript JSONL to Trash, and moves the
                generated folder to Trash. If the folder is already missing, deletion downgrades to
                session-only cleanup.
              </p>
              <p>
                Use Save first to copy the folder to Documents/Codex Saved Workspaces before moving
                the original to Trash.
              </p>
            {:else}
              <h3>Remove this workspace from Codex history?</h3>
              <p>
                This will delete all Codex sessions attached to this workspace, including transcript
                JSONL files and Codex database/index entries. It will not modify the workspace folder
                or project files.
              </p>
            {/if}
            <p class="confirmation-target" title={pendingTrashWorkspace.path}>
              {pendingTrashWorkspace.path}
            </p>
          </div>
          <div class="confirmation-actions">
            <button
              class="secondary-button"
              type="button"
              disabled={trashActionSessionId === workspaceTrashActionKey(pendingTrashWorkspace) ||
                trashActionSessionId === pendingTrashWorkspacePrimarySession?.id}
              onclick={clearPendingTrashAction}
            >
              Cancel
            </button>
            {#if pendingTrashWorkspace.source === "codexTaskFolder" && pendingTrashWorkspacePrimarySession}
              <button
                class="danger-button"
                type="button"
                disabled={trashActionSessionId === pendingTrashWorkspacePrimarySession.id}
                onclick={() =>
                  confirmGeneratedWorkspaceSessionTrash(
                    pendingTrashWorkspacePrimarySession.id,
                    false,
                  )}
              >
                {#if trashActionSessionId === pendingTrashWorkspacePrimarySession.id}
                  Working...
                {:else}
                  Delete session and folder
                {/if}
              </button>
              <button
                class="danger-button"
                type="button"
                disabled={trashActionSessionId === pendingTrashWorkspacePrimarySession.id}
                onclick={() =>
                  confirmGeneratedWorkspaceSessionTrash(
                    pendingTrashWorkspacePrimarySession.id,
                    true,
                  )}
              >
                {#if trashActionSessionId === pendingTrashWorkspacePrimarySession.id}
                  Working...
                {:else}
                  Save folder, then delete
                {/if}
              </button>
            {:else}
              <button
                class="danger-button"
                type="button"
                disabled={trashActionSessionId === workspaceTrashActionKey(pendingTrashWorkspace)}
                onclick={confirmWorkspaceHistoryTrash}
              >
                {#if trashActionSessionId === workspaceTrashActionKey(pendingTrashWorkspace)}
                  Working...
                {:else}
                  Remove workspace history
                {/if}
              </button>
            {/if}
          </div>
        </section>
      {/if}

      <dl class="facts-grid">
        <div>
          <dt>Workspace Path</dt>
          <dd title={selectedWorkspace.path}>{selectedWorkspace.path}</dd>
        </div>
        <div>
          <dt>Source</dt>
          <dd>{selectedWorkspace.sourceLabel}</dd>
        </div>
        <div>
          <dt>Sessions</dt>
          <dd>{formatCount(selectedWorkspace.sessionCount)}</dd>
        </div>
        <div>
          <dt>Active</dt>
          <dd>{formatCount(selectedWorkspace.activeCount)}</dd>
        </div>
        <div>
          <dt>Archived</dt>
          <dd>{formatCount(selectedWorkspace.archivedCount)}</dd>
        </div>
        <div>
          <dt>Last Used</dt>
          <dd>{selectedWorkspace.updatedAtLabel}</dd>
        </div>
      </dl>

      <section class="preview-card">
        <h3>Workspace Context</h3>
        <p>{workspaceSourceDescription(selectedWorkspace.source)}</p>
      </section>

      <section class="workspace-metadata">
        <div class="section-heading">
          <h3>Filesystem Metadata</h3>
          <span>{loadingWorkspacePath === selectedWorkspace.id ? "Loading" : "Lazy loaded"}</span>
        </div>

        {#if selectedWorkspaceMetadataError}
          <div class="state-panel error">{selectedWorkspaceMetadataError}</div>
        {:else if selectedWorkspaceMetadata}
          <dl class="metadata-grid">
            <div>
              <dt>Exists</dt>
              <dd>{selectedWorkspaceMetadata.exists ? "Yes" : "No"}</dd>
            </div>
            <div>
              <dt>Type</dt>
              <dd>
                {#if selectedWorkspaceMetadata.isDirectory}
                  Directory
                {:else if selectedWorkspaceMetadata.isFile}
                  File
                {:else}
                  Missing
                {/if}
              </dd>
            </div>
            <div>
              <dt>Size</dt>
              <dd>{formatSize(selectedWorkspaceMetadata.sizeBytes)}</dd>
            </div>
            <div>
              <dt>Files</dt>
              <dd>{formatCount(selectedWorkspaceMetadata.fileCount)}</dd>
            </div>
            <div>
              <dt>Folders</dt>
              <dd>{formatCount(selectedWorkspaceMetadata.directoryCount)}</dd>
            </div>
            <div>
              <dt>Modified</dt>
              <dd>
                {selectedWorkspaceMetadata.modifiedAtMs
                  ? formatDate(selectedWorkspaceMetadata.modifiedAtMs)
                  : "Unknown"}
              </dd>
            </div>
          </dl>

          {#if selectedWorkspaceMetadata.scanTruncated}
            <p class="metadata-note">
              Directory scan reached the safety limit, so size and counts are partial.
            </p>
          {/if}
        {:else}
          <div class="state-panel">Reading filesystem metadata for the selected workspace...</div>
        {/if}
      </section>

      <section class="related-sessions">
        <div class="section-heading">
          <h3>Sessions in this workspace</h3>
          <span>{formatCount(selectedWorkspace.sessionCount)}</span>
        </div>
        <div class="related-list">
          {#each selectedWorkspace.sessions as session (session.id)}
            <button class="related-session" type="button" onclick={() => selectSession(session)}>
              <span>{session.title}</span>
              <small>{session.updatedAtLabel} · {session.archived ? "Archived" : "Active"}</small>
            </button>
          {/each}
        </div>
      </section>
    {:else if selectedSession}
      <header class="detail-header">
        <div class="detail-heading">
          <p class="eyebrow">Selected session</p>
          <h2 title={selectedSession.title}>{selectedSession.title}</h2>
          {#if selectedSessionWorkspace}
            <p title={selectedSessionWorkspace.path}>{selectedSessionWorkspace.name}</p>
          {/if}
        </div>
        <div class="detail-actions" aria-label="Session actions">
          <button
            class="icon-button"
            type="button"
            aria-label="Open workspace folder"
            title="Open workspace folder"
            onclick={openSelectedWorkspaceFolder}
          >
            <FolderOpen size={17} />
          </button>
          <button
            class="icon-button"
            type="button"
            aria-label={selectedSession.archived ? "Restore session" : "Archive session"}
            title={selectedSession.archived ? "Restore session" : "Archive session"}
            disabled={archiveActionSessionId === selectedSession.id}
            onclick={requestSelectedSessionArchiveState}
          >
            <Archive size={17} />
          </button>
          <button
            class="icon-button danger"
            type="button"
            aria-label="Move to Trash"
            title="Move to Trash"
            disabled={trashActionSessionId === selectedSession.id}
            onclick={requestSelectedSessionTrash}
          >
            <Trash2 size={17} />
          </button>
        </div>
      </header>

      {#if archiveError}
        <div class="state-panel error action-error">{archiveError}</div>
      {/if}

      {#if trashError}
        <div class="state-panel error action-error">{trashError}</div>
      {/if}

      {#if pendingArchiveSession && pendingArchiveNextArchived !== null}
        <section class="confirmation-card" aria-label="Confirm archive action">
          <div>
            <h3>{pendingArchiveNextArchived ? "Archive this session?" : "Restore this session?"}</h3>
            <p>
              This will {pendingArchiveNextArchived ? "archive" : "restore"} the selected Codex
              session by moving its transcript file and updating Codex state_5.sqlite. For best
              results, close Codex Desktop before continuing.
            </p>
            <p class="confirmation-target" title={pendingArchiveSession.title}>
              {pendingArchiveSession.title}
            </p>
          </div>
          <div class="confirmation-actions">
            <button
              class="secondary-button"
              type="button"
              disabled={archiveActionSessionId === pendingArchiveSession.id}
              onclick={clearPendingArchiveAction}
            >
              Cancel
            </button>
            <button
              class="danger-button"
              type="button"
              disabled={archiveActionSessionId === pendingArchiveSession.id}
              onclick={confirmSelectedSessionArchiveState}
            >
              {#if archiveActionSessionId === pendingArchiveSession.id}
                Working...
              {:else}
                {pendingArchiveNextArchived ? "Archive" : "Restore"}
              {/if}
            </button>
          </div>
        </section>
      {/if}

      {#if pendingTrashSession}
        <section class="confirmation-card" aria-label="Confirm trash action">
          <div>
            {#if pendingTrashSessionIsGeneratedWorkspace}
              <h3>Delete this generated conversation?</h3>
              <p>
                This session uses a Codex-generated workspace folder, so deleting the session should
                also handle that generated folder. Choose whether to move the folder directly to
                Trash, or save a copy under Documents/Codex Saved Workspaces before moving the
                original to Trash. If the folder is already missing, deletion downgrades to
                session-only cleanup.
              </p>
            {:else}
              <h3>Move this session to Trash?</h3>
              <p>
                This moves the selected transcript JSONL to Trash, removes the matching thread row
                from Codex state_5.sqlite, clears known database references, and removes the matching
                session_index.jsonl entry. Project files are not modified. You can also remove the
                whole workspace from Codex history below without touching files.
              </p>
            {/if}
            <p class="confirmation-target" title={pendingTrashSession.title}>
              {pendingTrashSession.title}
            </p>
          </div>
          <div class="confirmation-actions">
            <button
              class="secondary-button"
              type="button"
              disabled={trashActionSessionId === pendingTrashSession.id}
              onclick={clearPendingTrashAction}
            >
              Cancel
            </button>
            {#if pendingTrashSessionIsGeneratedWorkspace}
              <button
                class="danger-button"
                type="button"
                disabled={trashActionSessionId === pendingTrashSession.id}
                onclick={() => confirmGeneratedWorkspaceSessionTrash(pendingTrashSession.id, false)}
              >
                {#if trashActionSessionId === pendingTrashSession.id}
                  Working...
                {:else}
                  Delete session and folder
                {/if}
              </button>
              <button
                class="danger-button"
                type="button"
                disabled={trashActionSessionId === pendingTrashSession.id}
                onclick={() => confirmGeneratedWorkspaceSessionTrash(pendingTrashSession.id, true)}
              >
                {#if trashActionSessionId === pendingTrashSession.id}
                  Working...
                {:else}
                  Save folder, then delete
                {/if}
              </button>
            {:else}
              <button
                class="danger-button"
                type="button"
                disabled={trashActionSessionId === pendingTrashSession.id}
                onclick={confirmSelectedSessionTrash}
              >
                {#if trashActionSessionId === pendingTrashSession.id}
                  Working...
                {:else}
                  Move session to Trash
                {/if}
              </button>
              <button
                class="danger-button"
                type="button"
                disabled={!pendingTrashSessionWorkspace ||
                  trashActionSessionId === workspaceTrashActionKey(pendingTrashSessionWorkspace)}
                onclick={confirmPendingSessionWorkspaceHistoryTrash}
              >
                {#if pendingTrashSessionWorkspace && trashActionSessionId === workspaceTrashActionKey(pendingTrashSessionWorkspace)}
                  Working...
                {:else}
                  Remove workspace history
                {/if}
              </button>
            {/if}
          </div>
        </section>
      {/if}

      <dl class="facts-grid">
        <div>
          <dt>Session ID</dt>
          <dd>{selectedSession.id}</dd>
        </div>
        <div>
          <dt>Status</dt>
          <dd>{selectedSession.archived ? "Archived" : "Active"}</dd>
        </div>
        <div>
          <dt>Workspace</dt>
          <dd title={selectedSession.cwd}>{selectedSession.cwd}</dd>
        </div>
        <div>
          <dt>Workspace Source</dt>
          <dd>{selectedSessionWorkspace?.sourceLabel ?? "Unknown"}</dd>
        </div>
        <div>
          <dt>Updated</dt>
          <dd>{selectedSession.updatedAtLabel}</dd>
        </div>
        <div>
          <dt>Model</dt>
          <dd>{selectedSession.model ?? "Unknown"}</dd>
        </div>
        <div class="wide-fact">
          <dt>Transcript</dt>
          <dd title={selectedSession.rolloutPath}>{selectedSession.rolloutPath}</dd>
        </div>
      </dl>

      <section class="preview-card">
        <h3>Preview</h3>
        <p>{selectedSession.preview}</p>
      </section>

      <section class="transcript">
        <div class="section-heading">
          <h3>Transcript</h3>
          <span>
            {#if selectedTranscriptIsLoading}
              Loading
            {:else if selectedTranscript?.truncated}
              Truncated
            {:else}
              On demand
            {/if}
          </span>
        </div>

        {#if selectedTranscriptIsLoading && !selectedTranscript}
          <div class="state-panel">Reading transcript JSONL for the selected session...</div>
        {:else if selectedTranscriptError}
          <div class="state-panel error">{selectedTranscriptError}</div>
        {:else if selectedTranscript}
          {#if !selectedTranscript.exists}
            <div class="state-panel">Transcript file was not found at {selectedTranscript.path}.</div>
          {:else}
            <dl class="metadata-grid transcript-stats">
              <div>
                <dt>Lines Read</dt>
                <dd>{formatCount(selectedTranscript.lineCount)}</dd>
              </div>
              <div>
                <dt>Messages</dt>
                <dd>{formatCount(selectedTranscript.messages.length)}</dd>
              </div>
              <div>
                <dt>Invalid Lines</dt>
                <dd>{formatCount(selectedTranscript.invalidLineCount)}</dd>
              </div>
            </dl>

            {#if selectedTranscript.truncated}
              <p class="metadata-note">
                Transcript parsing stopped at the safety limit, so this preview is partial.
              </p>
            {/if}

            <div class="transcript-filters" aria-label="Transcript filters">
              <label class="transcript-search">
                <Search size={16} />
                <input
                  bind:value={transcriptQuery}
                  aria-label="Search transcript messages"
                  type="search"
                  placeholder="Search transcript messages"
                />
              </label>

              <label class="role-filter">
                <span>Role</span>
                <select bind:value={transcriptRoleFilter}>
                  <option value="all">All</option>
                  {#each transcriptRoleOptions as role}
                    <option value={role}>{role}</option>
                  {/each}
                </select>
              </label>
            </div>

            <div class="filter-summary">
              {#if transcriptFilterActive}
                <span>
                  Showing {formatCount(filteredTranscriptMessages.length)} of {formatCount(
                    selectedTranscript.messages.length,
                  )} messages
                </span>
                <button
                  type="button"
                  onclick={() => {
                    transcriptQuery = "";
                    transcriptRoleFilter = "all";
                  }}
                >
                  Clear filters
                </button>
              {:else}
                <span>Showing all {formatCount(selectedTranscript.messages.length)} messages</span>
              {/if}
            </div>

            {#if selectedTranscript.messages.length === 0}
              <div class="state-panel">No displayable messages were found in this transcript.</div>
            {:else if filteredTranscriptMessages.length === 0}
              <div class="state-panel">No transcript messages match the current filters.</div>
            {:else}
              <div class="transcript-list" aria-label="Transcript messages">
                {#each filteredTranscriptMessages as message}
                  <article class="transcript-message">
                    <div class="message-meta">
                      <span class="message-role">{message.role}</span>
                      <span>Line {message.lineNumber}</span>
                      {#if message.timestamp}
                        <time datetime={message.timestamp}>{message.timestamp}</time>
                      {/if}
                    </div>
                    <pre class="transcript-text">{message.text}</pre>
                  </article>
                {/each}
              </div>
            {/if}
          {/if}
        {:else}
          <div class="state-panel">Transcript has not loaded yet.</div>
        {/if}
      </section>
    {:else if historyFilter === "settings"}
      <section class="settings-detail" aria-label="Settings details">
        <p class="eyebrow">Settings</p>
        <h2>Local-first configuration</h2>
        <p>
          There are no editable preferences yet. This app currently reads Codex history locally and
          only writes when you explicitly confirm archive, restore, or trash operations.
        </p>
      </section>
    {:else}
      <div class="empty-detail">Select a project, conversation, or session to inspect metadata.</div>
    {/if}
  </section>
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(html) {
    height: 100%;
    overflow: hidden;
  }

  :global(body) {
    margin: 0;
    height: 100%;
    min-width: 980px;
    overflow: hidden;
    color: #1a1f24;
    background: #f5f7f8;
    font-family:
      Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    letter-spacing: 0;
  }

  :global(button),
  :global(input),
  :global(select) {
    font: inherit;
  }

  :global(button) {
    color: inherit;
  }

  :global(h1),
  :global(h2),
  :global(h3),
  :global(p) {
    margin-top: 0;
  }

  .app-shell {
    display: grid;
    grid-template-columns: 248px minmax(360px, 460px) minmax(520px, 1fr);
    height: 100vh;
    min-height: 0;
    overflow: hidden;
    background: #f5f7f8;
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 22px;
    min-height: 0;
    overflow: hidden;
    padding: 22px 18px;
    border-right: 1px solid #d9dee3;
    background: #eef2f4;
  }

  .brand,
  .storage-note {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .brand-mark {
    display: grid;
    width: 38px;
    height: 38px;
    place-items: center;
    border: 1px solid #c8d0d7;
    border-radius: 9px;
    color: #0d5c63;
    background: #ffffff;
  }

  .brand strong,
  .storage-note strong {
    display: block;
    font-size: 14px;
    line-height: 20px;
  }

  .brand span,
  .storage-note span {
    display: block;
    color: #66717c;
    font-size: 12px;
    line-height: 17px;
  }

  .library-card,
  .storage-note {
    border: 1px solid #d0d8de;
    border-radius: 10px;
    background: #ffffff;
  }

  .library-card {
    padding: 14px;
  }

  .library-card p:not(.eyebrow) {
    margin-bottom: 0;
    color: #53616d;
    font-size: 13px;
    line-height: 20px;
  }

  .nav-list {
    display: grid;
    gap: 6px;
  }

  .nav-item {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    width: 100%;
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    border: 0;
    border-radius: 8px;
    color: #43505c;
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  .nav-item:hover,
  .nav-item.active {
    color: #142027;
    background: #ffffff;
  }

  .nav-item span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .nav-item small {
    color: #71808b;
    font-size: 12px;
    line-height: 17px;
  }

  .storage-note {
    margin-top: auto;
    align-items: flex-start;
    padding: 12px;
    color: #0d5c63;
  }

  .history-panel,
  .detail-panel {
    min-height: 0;
    overflow-y: auto;
  }

  .history-panel {
    padding: 22px 18px;
    border-right: 1px solid #d9dee3;
    background: #f9fbfc;
  }

  .detail-panel {
    display: grid;
    align-content: start;
    gap: 22px;
    padding: 24px;
  }

  .panel-header,
  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .panel-header {
    margin-bottom: 16px;
  }

  .panel-description {
    margin-bottom: 4px;
    color: #52606c;
    font-size: 13px;
    line-height: 20px;
  }

  .detail-heading {
    min-width: 0;
  }

  .eyebrow {
    margin-bottom: 4px;
    color: #0d5c63;
    font-size: 11px;
    font-weight: 750;
    letter-spacing: 0.08em;
    line-height: 16px;
    text-transform: uppercase;
  }

  h1,
  h2,
  h3 {
    color: #162128;
  }

  h1 {
    margin-bottom: 4px;
    font-size: 25px;
    line-height: 32px;
  }

  h2 {
    overflow: hidden;
    margin-bottom: 4px;
    font-size: 22px;
    line-height: 29px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  h3 {
    margin-bottom: 8px;
    font-size: 15px;
    line-height: 21px;
  }

  .home-path,
  .detail-heading p:not(.eyebrow) {
    overflow: hidden;
    max-width: 100%;
    margin-bottom: 0;
    color: #697681;
    font-size: 12px;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .icon-button {
    display: inline-grid;
    flex: 0 0 auto;
    width: 34px;
    height: 34px;
    place-items: center;
    border: 1px solid #cfd8de;
    border-radius: 8px;
    color: #35414c;
    background: #ffffff;
    cursor: pointer;
  }

  .icon-button:hover {
    border-color: #9eb1bd;
    background: #f4fafb;
  }

  .icon-button.danger {
    color: #9a372f;
  }

  .icon-button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .search-box,
  .transcript-search,
  .role-filter {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    border: 1px solid #dce4e8;
    border-radius: 8px;
    color: #6c7782;
    background: #ffffff;
  }

  .search-box {
    padding: 0 11px;
  }

  .search-box input,
  .transcript-search input,
  .role-filter select {
    width: 100%;
    min-width: 0;
    height: 38px;
    border: 0;
    outline: 0;
    color: #192229;
    background: transparent;
  }

  .list-meta {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    margin: 12px 0 14px;
    color: #68747f;
    font-size: 12px;
    line-height: 18px;
  }

  .workspace-list,
  .workspace-stack,
  .nested-sessions,
  .related-list,
  .transcript-list {
    display: grid;
    gap: 10px;
  }

  .workspace-section {
    display: grid;
    gap: 9px;
  }

  .workspace-section + .workspace-section {
    margin-top: 20px;
  }

  .workspace-section-heading {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 10px;
    padding: 0 2px;
  }

  .workspace-section-heading h2 {
    margin-bottom: 1px;
    font-size: 13px;
    letter-spacing: 0.08em;
    line-height: 18px;
    text-transform: uppercase;
  }

  .workspace-section-heading p {
    margin-bottom: 0;
    color: #79848e;
    font-size: 12px;
    line-height: 17px;
  }

  .workspace-section-heading span {
    color: #79848e;
    font-size: 12px;
    line-height: 18px;
  }

  .workspace-card {
    overflow: hidden;
    border: 1px solid #dfe6ea;
    border-radius: 11px;
    background: #ffffff;
  }

  .workspace-card.active {
    border-color: #8bb5bc;
    box-shadow: 0 0 0 1px #8bb5bc inset;
  }

  .workspace-card-header {
    display: grid;
    gap: 6px;
    width: 100%;
    padding: 13px;
    border: 0;
    border-bottom: 1px solid #edf1f3;
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  .workspace-card-header:hover {
    background: #f5fafb;
  }

  .workspace-title-line {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 8px;
  }

  .workspace-name,
  .session-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .workspace-name {
    font-size: 14px;
    font-weight: 750;
    line-height: 20px;
  }

  .workspace-badge {
    flex: 0 0 auto;
    padding: 2px 6px;
    border: 1px solid #d7e1e5;
    border-radius: 999px;
    color: #60707b;
    background: #f5f8f9;
    font-size: 11px;
    font-weight: 700;
    line-height: 15px;
  }

  .workspace-path {
    overflow: hidden;
    color: #52606c;
    font-size: 12px;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-footer {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    color: #79848e;
    font-size: 12px;
    line-height: 18px;
  }

  .nested-sessions {
    padding: 8px;
    background: #fbfcfd;
  }

  .nested-session-row,
  .related-session {
    display: grid;
    gap: 4px;
    width: 100%;
    border: 1px solid #e5ebef;
    border-radius: 9px;
    color: inherit;
    background: #ffffff;
    text-align: left;
    cursor: pointer;
  }

  .nested-session-row {
    padding: 10px;
  }

  .nested-session-row:hover,
  .nested-session-row.active,
  .related-session:hover {
    border-color: #9eb1bd;
    background: #f4fafb;
  }

  .session-row-main {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 8px;
  }

  .session-dot {
    flex: 0 0 auto;
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: #2f8f5b;
  }

  .session-dot.archived {
    background: #a8783c;
  }

  .session-title {
    font-size: 13px;
    font-weight: 700;
    line-height: 19px;
  }

  .session-preview {
    display: -webkit-box;
    overflow: hidden;
    color: #5b6873;
    font-size: 12px;
    line-height: 18px;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
  }

  .state-panel,
  .empty-detail {
    padding: 16px;
    border: 1px solid #dbe1e5;
    border-radius: 9px;
    color: #53616d;
    background: #f8fafb;
    font-size: 13px;
    line-height: 20px;
  }

  .state-panel.error {
    border-color: #e3b7b2;
    color: #8c312b;
    background: #fff7f6;
  }

  .action-error {
    margin: 0;
  }

  .detail-actions {
    display: flex;
    flex-shrink: 0;
    gap: 8px;
  }

  .confirmation-card {
    display: grid;
    gap: 14px;
    padding: 16px;
    border: 1px solid #e2b8a5;
    border-radius: 9px;
    background: #fff8f3;
  }

  .confirmation-card h3 {
    color: #7b351c;
  }

  .confirmation-card p {
    margin-bottom: 0;
    color: #65483b;
    font-size: 13px;
    line-height: 20px;
  }

  .confirmation-target {
    overflow: hidden;
    max-width: 100%;
    margin-top: 8px;
    font-weight: 750;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .confirmation-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 8px;
  }

  .secondary-button,
  .danger-button {
    min-height: 34px;
    padding: 0 12px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 700;
  }

  .secondary-button {
    border: 1px solid #cfd7dd;
    color: #35414c;
    background: #ffffff;
  }

  .danger-button {
    border: 1px solid #b85042;
    color: #ffffff;
    background: #a33a32;
  }

  .secondary-button:disabled,
  .danger-button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .facts-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
    margin: 0;
  }

  .facts-grid div,
  .preview-card,
  .settings-detail,
  .settings-panel,
  .workspace-metadata,
  .related-sessions,
  .transcript {
    border: 1px solid #dbe1e5;
    border-radius: 9px;
    background: #ffffff;
  }

  .facts-grid div {
    min-width: 0;
    padding: 13px;
  }

  .facts-grid .wide-fact {
    grid-column: 1 / -1;
  }

  dt {
    margin-bottom: 6px;
    color: #68747f;
    font-size: 12px;
    line-height: 17px;
  }

  dd {
    overflow: hidden;
    margin: 0;
    font-size: 13px;
    line-height: 20px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preview-card,
  .settings-detail,
  .settings-panel,
  .workspace-metadata,
  .related-sessions,
  .transcript {
    padding: 17px;
  }

  .preview-card p,
  .settings-detail p,
  .settings-panel p {
    margin-bottom: 0;
    color: #44515d;
    line-height: 24px;
  }

  .settings-list {
    display: grid;
    gap: 10px;
    margin: 14px 0 0;
  }

  .settings-list div {
    min-width: 0;
    padding: 11px;
    border: 1px solid #e0e6ea;
    border-radius: 8px;
    background: #f9fbfc;
  }

  .section-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }

  .section-heading h3 {
    margin-bottom: 0;
  }

  .section-heading span {
    color: #71808b;
    font-size: 12px;
    line-height: 17px;
  }

  .metadata-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
    margin: 0;
  }

  .metadata-grid div {
    min-width: 0;
    padding: 11px;
    border: 1px solid #e0e6ea;
    border-radius: 8px;
    background: #f9fbfc;
  }

  .metadata-note {
    margin: 12px 0 0;
    color: #68747f;
    font-size: 12px;
    line-height: 18px;
  }

  .related-session {
    padding: 11px 12px;
  }

  .related-session span,
  .related-session small {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .related-session span {
    font-size: 13px;
    font-weight: 700;
    line-height: 19px;
  }

  .related-session small {
    color: #71808b;
    font-size: 12px;
    line-height: 17px;
  }

  .transcript-stats {
    margin-bottom: 14px;
  }

  .transcript-filters {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(140px, 180px);
    gap: 10px;
    margin-top: 14px;
  }

  .transcript-search,
  .role-filter {
    padding: 0 10px;
    background: #f8fafb;
  }

  .role-filter span {
    color: #71808b;
    font-size: 12px;
    line-height: 17px;
  }

  .filter-summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 10px;
    color: #71808b;
    font-size: 12px;
    line-height: 17px;
  }

  .filter-summary button {
    padding: 0;
    border: 0;
    color: #0d5c63;
    background: transparent;
    cursor: pointer;
    font-weight: 700;
  }

  .filter-summary button:hover {
    text-decoration: underline;
  }

  .transcript-list {
    margin-top: 14px;
  }

  .transcript-message {
    display: grid;
    gap: 8px;
    padding: 12px;
    border: 1px solid #e0e6ea;
    border-radius: 8px;
    background: #f9fbfc;
  }

  .message-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 7px;
    color: #71808b;
    font-size: 12px;
    line-height: 17px;
  }

  .message-role {
    color: #0d5c63;
    font-weight: 750;
    text-transform: capitalize;
  }

  .transcript-text {
    overflow-x: auto;
    margin: 0;
    color: #28343d;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
    font-size: 12px;
    line-height: 19px;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
