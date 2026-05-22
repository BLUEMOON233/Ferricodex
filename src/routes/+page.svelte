<script lang="ts">
  import {
    Archive,
    Database,
    FileSearch,
    FolderOpen,
    History,
    Play,
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
    workspaceSourceDescription,
    type ViewMode,
    type Workspace,
  } from "$lib/workspace";
  import { openLocalPath } from "$lib/opener";

  let query = $state("");
  let activeView = $state<ViewMode>("sessions");
  let selectedId = $state("");
  let selectedWorkspaceId = $state("");
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

  function clearPendingArchiveAction() {
    pendingArchiveSessionId = "";
    pendingArchiveNextArchived = null;
  }

  function requestSelectedSessionArchiveState() {
    if (!selectedSession) {
      archiveError = "No session is selected.";
      return;
    }

    pendingArchiveSessionId = selectedSession.id;
    pendingArchiveNextArchived = !selectedSession.archived;
    archiveError = "";
  }

  async function confirmSelectedSessionArchiveState() {
    if (!pendingArchiveSessionId || pendingArchiveNextArchived === null) {
      archiveError = "No archive action is pending.";
      return;
    }

    const sessionId = pendingArchiveSessionId;
    const nextArchived = pendingArchiveNextArchived;
    const action = nextArchived ? "archive" : "unarchive";

    archiveActionSessionId = sessionId;
    archiveError = "";

    try {
      await setThreadArchiveState(sessionId, nextArchived);

      await loadSessions();
      activeView = nextArchived ? "archive" : "sessions";
      selectedId = sessionId;
    } catch (error) {
      archiveError = `Could not ${action} session: ${errorMessage(error)}`;
    } finally {
      archiveActionSessionId = "";
      clearPendingArchiveAction();
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
      selectedId = sessions[0]?.id ?? "";
      selectedWorkspaceId = "";
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
      transcriptQuery = "";
      transcriptRoleFilter = "all";
    } catch (error) {
      loadError = errorMessage(error);
      sessions = [];
      selectedId = "";
      selectedWorkspaceId = "";
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
      transcriptQuery = "";
      transcriptRoleFilter = "all";
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    loadSessions();
  });

  const searchedSessions = $derived(
    sessions.filter((session) => {
      const value = `${session.title} ${session.cwd} ${session.preview} ${session.id}`.toLowerCase();
      return value.includes(query.trim().toLowerCase());
    }),
  );

  const visibleSessions = $derived(
    searchedSessions.filter((session) =>
      activeView === "archive" ? session.archived : !session.archived,
    ),
  );

  const activeSessionCount = $derived(sessions.filter((session) => !session.archived).length);
  const archivedSessionCount = $derived(sessions.filter((session) => session.archived).length);

  const currentViewTitle = $derived(
    activeView === "sessions" ? "Sessions" : activeView === "workspaces" ? "Workspaces" : "Archive",
  );

  const workspaces = $derived.by((): Workspace[] =>
    groupSessionsByWorkspace(sessions, codexHome?.path),
  );

  const filteredWorkspaces = $derived(
    workspaces.filter((workspace) => {
      const sessionText = workspace.sessions
        .map((session) => `${session.title} ${session.preview} ${session.id}`)
        .join(" ");
      const value =
        `${workspace.name} ${workspace.path} ${workspace.sourceLabel} ${sessionText}`.toLowerCase();

      return value.includes(query.trim().toLowerCase());
    }),
  );

  const filteredWorkspaceSessionCount = $derived(
    filteredWorkspaces.reduce((total, workspace) => total + workspace.sessionCount, 0),
  );

  const selectedSession = $derived(
    visibleSessions.find((session) => session.id === selectedId) ?? visibleSessions[0] ?? null,
  );

  const pendingArchiveSession = $derived(
    pendingArchiveSessionId
      ? (sessions.find((session) => session.id === pendingArchiveSessionId) ?? null)
      : null,
  );

  const selectedWorkspace = $derived(
    filteredWorkspaces.find((workspace) => workspace.id === selectedWorkspaceId) ??
      filteredWorkspaces[0] ??
      null,
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

  const selectedWorkspaceMetadata = $derived(
    selectedWorkspace ? (workspaceMetadataByPath[selectedWorkspace.id] ?? null) : null,
  );

  const selectedWorkspaceMetadataError = $derived(
    selectedWorkspace ? (workspaceMetadataErrorsByPath[selectedWorkspace.id] ?? "") : "",
  );

  $effect(() => {
    if (activeView !== "workspaces" || !selectedWorkspace) {
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
          [workspaceId]: error instanceof Error ? error.message : String(error),
        };
      })
      .finally(() => {
        if (loadingWorkspacePath === workspaceId) {
          loadingWorkspacePath = "";
        }
      });
  });

  $effect(() => {
    if ((activeView !== "sessions" && activeView !== "archive") || !selectedSession) {
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
        class:active={activeView === "sessions"}
        class="nav-item"
        type="button"
        onclick={() => (activeView = "sessions")}
      >
        <Database size={17} />
        Sessions
      </button>
      <button
        class:active={activeView === "workspaces"}
        class="nav-item"
        type="button"
        onclick={() => (activeView = "workspaces")}
      >
        <FolderOpen size={17} />
        Workspaces
      </button>
      <button
        class:active={activeView === "archive"}
        class="nav-item"
        type="button"
        onclick={() => (activeView = "archive")}
      >
        <Archive size={17} />
        Archive
      </button>
      <button class="nav-item" type="button">
        <Settings size={17} />
        Settings
      </button>
    </nav>

    <section class="storage-note" aria-label="Storage policy">
      <ShieldCheck size={18} />
      <div>
        <strong>Codex-safe design</strong>
        <span>Writes are explicit, confirmed, and scoped to Codex-compatible actions.</span>
      </div>
    </section>
  </aside>

  <section class="session-list" aria-label="Codex sessions">
    <header class="panel-header">
      <div>
        <p class="eyebrow">Local Codex store</p>
        <h1>{currentViewTitle}</h1>
        {#if codexHome?.path}
          <p class="home-path" title={codexHome.path}>{codexHome.path}</p>
        {/if}
      </div>
      <button
        class="icon-button"
        type="button"
        aria-label="Open Codex data directory"
        onclick={openCodexHomeDirectory}
      >
        <FolderOpen size={18} />
      </button>
    </header>

    <label class="search-box">
      <Search size={18} />
      <input
        bind:value={query}
        type="search"
        placeholder={activeView === "workspaces"
          ? "Search workspace, path, source, or session"
          : "Search title, path, preview, or id"}
      />
    </label>

    <div class="list-meta">
      {#if activeView === "workspaces"}
        <span>{filteredWorkspaces.length} workspaces / {filteredWorkspaceSessionCount} sessions</span>
      {:else if activeView === "archive"}
        <span>{visibleSessions.length} archived sessions / {archivedSessionCount} total</span>
      {:else}
        <span>{visibleSessions.length} active sessions / {activeSessionCount} total</span>
      {/if}
      <span>{isLoading ? "Loading" : "Ready"}</span>
    </div>

    {#if openerError}
      <div class="state-panel error action-error">{openerError}</div>
    {/if}

    <div class="sessions" aria-live="polite">
      {#if isLoading}
        <div class="state-panel">Reading local Codex history...</div>
      {:else if loadError}
        <div class="state-panel error">{loadError}</div>
      {:else if (activeView === "sessions" || activeView === "archive") && visibleSessions.length === 0}
        <div class="state-panel">
          {activeView === "archive"
            ? "No archived sessions match the current search."
            : "No active sessions match the current search."}
        </div>
      {:else if activeView === "workspaces" && filteredWorkspaces.length === 0}
        <div class="state-panel">No workspaces match the current search.</div>
      {:else if activeView === "sessions" || activeView === "archive"}
        {#each visibleSessions as session}
          <button
            class:active={session.id === selectedSession?.id}
            class="session-row"
            type="button"
            onclick={() => (selectedId = session.id)}
          >
            <span class="row-title">{session.title}</span>
            <span class="row-preview">{session.preview}</span>
            <span class="row-footer">
              <span>{session.updatedAtLabel}</span>
              <span>{session.archived ? "Archived" : (session.model ?? "model unknown")}</span>
            </span>
          </button>
        {/each}
      {:else}
        {#each filteredWorkspaces as workspace}
          <button
            class:active={workspace.id === selectedWorkspace?.id}
            class="workspace-row"
            type="button"
            onclick={() => (selectedWorkspaceId = workspace.id)}
          >
            <span class="row-title">{workspace.name}</span>
            <span class="row-path" title={workspace.path}>{workspace.path}</span>
            <span class="row-footer">
              <span>{workspace.sessionCount} sessions</span>
              <span>{workspace.sourceLabel}</span>
            </span>
            <span class="row-footer">
              <span>Last used {workspace.updatedAtLabel}</span>
              <span>{workspace.archivedCount} archived</span>
            </span>
          </button>
        {/each}
      {/if}
    </div>
  </section>

  <section class="detail-panel" aria-label="Session details">
    {#if activeView === "workspaces" && selectedWorkspace}
      <header class="detail-header">
        <div class="detail-heading">
          <p class="eyebrow">Selected workspace</p>
          <h2 title={selectedWorkspace.name}>{selectedWorkspace.name}</h2>
        </div>
        <div class="detail-actions" aria-label="Workspace actions">
          <button
            class="icon-button"
            type="button"
            aria-label="Open workspace folder"
            onclick={openSelectedWorkspaceFolder}
          >
            <FolderOpen size={17} />
          </button>
        </div>
      </header>

      <dl class="session-facts">
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
          <dd>{selectedWorkspace.sessionCount}</dd>
        </div>
        <div>
          <dt>Active</dt>
          <dd>{selectedWorkspace.activeCount}</dd>
        </div>
        <div>
          <dt>Archived</dt>
          <dd>{selectedWorkspace.archivedCount}</dd>
        </div>
        <div>
          <dt>Last Used</dt>
          <dd>{selectedWorkspace.updatedAtLabel}</dd>
        </div>
      </dl>

      <section class="preview">
        <h3>Workspace Context</h3>
        <p>{workspaceSourceDescription(selectedWorkspace.source)}</p>
      </section>

      <section class="workspace-metadata">
        <div class="section-heading">
          <h3>Filesystem Metadata</h3>
          <span>
            {#if loadingWorkspacePath === selectedWorkspace.id}
              Loading
            {:else}
              Lazy loaded
            {/if}
          </span>
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
        <h3>Related Sessions</h3>
        <div class="related-list">
          {#each selectedWorkspace.sessions as session}
            <button
              class="related-session"
              type="button"
              onclick={() => {
                activeView = session.archived ? "archive" : "sessions";
                selectedId = session.id;
              }}
            >
              <span>{session.title}</span>
              <small>{session.updatedAtLabel}</small>
            </button>
          {/each}
        </div>
      </section>
    {:else if (activeView === "sessions" || activeView === "archive") && selectedSession}
      <header class="detail-header">
        <div class="detail-heading">
          <p class="eyebrow">{selectedSession.archived ? "Archived session" : "Selected session"}</p>
          <h2 title={selectedSession.title}>{selectedSession.title}</h2>
        </div>
        <div class="detail-actions" aria-label="Session actions">
          <button class="icon-button" type="button" aria-label="Resume session">
            <Play size={17} />
          </button>
          <button
            class="icon-button"
            type="button"
            aria-label="Reload transcript"
            onclick={() => selectedSession && loadTranscriptForSession(selectedSession, true)}
          >
            <FileSearch size={17} />
          </button>
          <button
            class="icon-button"
            type="button"
            aria-label={selectedSession.archived ? "Unarchive session" : "Archive session"}
            title={selectedSession.archived ? "Unarchive session" : "Archive session"}
            disabled={archiveActionSessionId === selectedSession.id}
            onclick={requestSelectedSessionArchiveState}
          >
            <Archive size={17} />
          </button>
          <button class="icon-button danger" type="button" aria-label="Move to trash">
            <Trash2 size={17} />
          </button>
        </div>
      </header>

      {#if archiveError}
        <div class="state-panel error action-error">{archiveError}</div>
      {/if}

      {#if pendingArchiveSession && pendingArchiveNextArchived !== null}
        <section class="archive-confirmation" aria-label="Confirm archive action">
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

      <dl class="session-facts">
        <div>
          <dt>Session ID</dt>
          <dd>{selectedSession.id}</dd>
        </div>
        <div>
          <dt>Project Path</dt>
          <dd>{selectedSession.cwd}</dd>
        </div>
        <div>
          <dt>Updated</dt>
          <dd>{selectedSession.updatedAtLabel}</dd>
        </div>
        <div>
          <dt>Status</dt>
          <dd>{selectedSession.archived ? "Archived" : "Active"}</dd>
        </div>
        <div>
          <dt>Transcript</dt>
          <dd>{selectedSession.rolloutPath}</dd>
        </div>
        <div>
          <dt>Model</dt>
          <dd>{selectedSession.model ?? "Unknown"}</dd>
        </div>
      </dl>

      <section class="preview">
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
    {:else}
      <div class="empty-detail">
        Select a {activeView === "workspaces" ? "workspace" : "session"} to inspect its local
        metadata.
      </div>
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
    min-width: 960px;
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

  .app-shell {
    display: grid;
    grid-template-columns: 248px minmax(320px, 420px) minmax(480px, 1fr);
    height: 100vh;
    min-height: 0;
    overflow: hidden;
    background: #f5f7f8;
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 26px;
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
    border-radius: 8px;
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

  .nav-list {
    display: grid;
    gap: 6px;
  }

  .nav-item {
    display: flex;
    width: 100%;
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    border: 0;
    border-radius: 7px;
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

  .storage-note {
    margin-top: auto;
    align-items: flex-start;
    padding: 12px;
    border: 1px solid #d0d8de;
    border-radius: 8px;
    color: #0d5c63;
    background: #ffffff;
  }

  .session-list,
  .detail-panel {
    min-width: 0;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    padding: 24px;
  }

  .session-list {
    border-right: 1px solid #d9dee3;
    background: #ffffff;
  }

  .panel-header,
  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .eyebrow {
    margin: 0 0 3px;
    color: #68747f;
    font-size: 12px;
    font-weight: 650;
    line-height: 16px;
    text-transform: uppercase;
  }

  h1,
  h2,
  h3,
  p {
    margin-top: 0;
  }

  h1 {
    margin-bottom: 0;
    font-size: 28px;
    line-height: 34px;
  }

  .home-path {
    overflow: hidden;
    max-width: 320px;
    margin: 4px 0 0;
    color: #68747f;
    font-size: 12px;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  h2 {
    margin-bottom: 0;
    font-size: 24px;
    line-height: 31px;
  }

  .detail-heading {
    min-width: 0;
  }

  .detail-heading h2 {
    display: -webkit-box;
    overflow: hidden;
    max-width: 100%;
    line-clamp: 2;
    overflow-wrap: anywhere;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
  }

  h3 {
    margin-bottom: 12px;
    font-size: 15px;
    line-height: 22px;
  }

  .icon-button {
    display: grid;
    width: 36px;
    height: 36px;
    place-items: center;
    border: 1px solid #d2d9df;
    border-radius: 7px;
    color: #35414c;
    background: #ffffff;
    cursor: pointer;
  }

  .icon-button:hover {
    border-color: #9eb1bd;
    background: #f7f9fa;
  }

  .icon-button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .icon-button:disabled:hover {
    border-color: #d2d9df;
    background: #ffffff;
  }

  .icon-button.danger {
    color: #a33a32;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-top: 22px;
    padding: 0 12px;
    border: 1px solid #ced6dc;
    border-radius: 7px;
    color: #6c7782;
    background: #f8fafb;
  }

  .search-box input {
    width: 100%;
    min-width: 0;
    height: 42px;
    border: 0;
    outline: 0;
    color: #192229;
    background: transparent;
  }

  .list-meta {
    display: flex;
    justify-content: space-between;
    margin: 15px 0 10px;
    color: #68747f;
    font-size: 12px;
  }

  .sessions {
    display: grid;
    gap: 8px;
  }

  .state-panel,
  .empty-detail {
    padding: 16px;
    border: 1px solid #dbe1e5;
    border-radius: 8px;
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
    margin: 12px 0;
  }

  .archive-confirmation {
    display: grid;
    gap: 14px;
    padding: 16px;
    border: 1px solid #e2b8a5;
    border-radius: 8px;
    background: #fff8f3;
  }

  .archive-confirmation h3 {
    margin-bottom: 6px;
    color: #7b351c;
  }

  .archive-confirmation p {
    margin-bottom: 0;
    color: #65483b;
    font-size: 13px;
    line-height: 20px;
  }

  .confirmation-target {
    overflow: hidden;
    max-width: 100%;
    margin-top: 8px;
    font-weight: 700;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .confirmation-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .secondary-button,
  .danger-button {
    min-height: 34px;
    padding: 0 12px;
    border-radius: 7px;
    cursor: pointer;
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

  .session-row {
    display: grid;
    gap: 5px;
    width: 100%;
    min-height: 104px;
    padding: 13px;
    border: 1px solid #e0e5e9;
    border-radius: 8px;
    color: inherit;
    background: #ffffff;
    text-align: left;
    cursor: pointer;
  }

  .workspace-row {
    display: grid;
    gap: 5px;
    width: 100%;
    min-height: 116px;
    padding: 13px;
    border: 1px solid #e0e5e9;
    border-radius: 8px;
    color: inherit;
    background: #ffffff;
    text-align: left;
    cursor: pointer;
  }

  .session-row:hover,
  .session-row.active,
  .workspace-row:hover,
  .workspace-row.active {
    border-color: #8bb5bc;
    background: #f4fafb;
  }

  .row-title {
    overflow: hidden;
    font-size: 14px;
    font-weight: 700;
    line-height: 20px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-path {
    overflow: hidden;
    color: #52606c;
    font-size: 12px;
    line-height: 18px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-preview {
    display: -webkit-box;
    overflow: hidden;
    min-height: 40px;
    color: #52606c;
    font-size: 13px;
    line-clamp: 2;
    line-height: 20px;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
  }

  .row-footer {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    color: #79848e;
    font-size: 12px;
    line-height: 18px;
  }

  .detail-panel {
    display: grid;
    align-content: start;
    gap: 22px;
  }

  .detail-actions {
    display: flex;
    flex-shrink: 0;
    gap: 8px;
  }

  .session-facts {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px;
    margin: 0;
  }

  .session-facts div,
  .preview,
  .workspace-metadata,
  .related-sessions,
  .transcript {
    border: 1px solid #dbe1e5;
    border-radius: 8px;
    background: #ffffff;
  }

  .session-facts div {
    min-width: 0;
    padding: 13px;
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

  .preview,
  .workspace-metadata,
  .related-sessions,
  .transcript {
    padding: 17px;
  }

  .preview p {
    margin-bottom: 0;
    color: #44515d;
    line-height: 24px;
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
    border-radius: 7px;
    background: #f9fbfc;
  }

  .metadata-note {
    margin: 12px 0 0;
    color: #68747f;
    font-size: 12px;
    line-height: 18px;
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
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    padding: 0 10px;
    border: 1px solid #dce4e8;
    border-radius: 7px;
    color: #6c7782;
    background: #f8fafb;
  }

  .transcript-search input,
  .role-filter select {
    width: 100%;
    min-width: 0;
    height: 36px;
    border: 0;
    outline: 0;
    color: #192229;
    background: transparent;
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
    font-weight: 650;
  }

  .filter-summary button:hover {
    text-decoration: underline;
  }

  .transcript-list {
    display: grid;
    gap: 10px;
    margin-top: 14px;
  }

  .transcript-message {
    display: grid;
    gap: 8px;
    padding: 12px;
    border: 1px solid #e0e6ea;
    border-radius: 7px;
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
    font-weight: 700;
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

  .related-list {
    display: grid;
    gap: 8px;
  }

  .related-session {
    display: grid;
    gap: 3px;
    width: 100%;
    padding: 11px 12px;
    border: 1px solid #e0e6ea;
    border-radius: 7px;
    color: inherit;
    background: #f9fbfc;
    text-align: left;
    cursor: pointer;
  }

  .related-session:hover {
    border-color: #9eb1bd;
    background: #f4fafb;
  }

  .related-session span,
  .related-session small {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .related-session span {
    font-size: 13px;
    font-weight: 650;
    line-height: 19px;
  }

  .related-session small {
    color: #71808b;
    font-size: 12px;
    line-height: 17px;
  }

</style>
