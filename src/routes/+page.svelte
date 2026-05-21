<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
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

  type Session = {
    id: string;
    title: string;
    cwd: string;
    updatedAt: number;
    updatedAtLabel: string;
    preview: string;
    model: string | null;
    archived: boolean;
    rolloutPath: string;
  };

  type ViewMode = "sessions" | "workspaces";
  type WorkspaceSource = "codexTaskFolder" | "codexWorktree" | "userProject";

  type Workspace = {
    id: string;
    name: string;
    path: string;
    source: WorkspaceSource;
    sourceLabel: string;
    sessions: Session[];
    sessionCount: number;
    activeCount: number;
    archivedCount: number;
    updatedAt: number;
    updatedAtLabel: string;
  };

  type WorkspaceMetadata = {
    path: string;
    exists: boolean;
    isDirectory: boolean;
    isFile: boolean;
    sizeBytes: number | null;
    fileCount: number | null;
    directoryCount: number | null;
    modifiedAtMs: number | null;
    scanTruncated: boolean;
  };

  type CodexThread = {
    id: string;
    title: string;
    cwd: string;
    preview: string;
    rolloutPath: string;
    createdAt: number;
    updatedAt: number;
    createdAtMs: number;
    updatedAtMs: number;
    model: string | null;
    archived: boolean;
  };

  type CodexHomeStatus = {
    path: string;
    exists: boolean;
    stateDbExists: boolean;
    source: "env" | "default";
  };

  let query = $state("");
  let activeView = $state<ViewMode>("sessions");
  let selectedId = $state("");
  let selectedWorkspaceId = $state("");
  let sessions = $state<Session[]>([]);
  let codexHome = $state<CodexHomeStatus | null>(null);
  let workspaceMetadataByPath = $state<Record<string, WorkspaceMetadata>>({});
  let workspaceMetadataErrorsByPath = $state<Record<string, string>>({});
  let loadingWorkspacePath = $state("");
  let isLoading = $state(true);
  let loadError = $state("");

  function formatDate(timestampMs: number) {
    return new Intl.DateTimeFormat(undefined, {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    }).format(new Date(timestampMs));
  }

  function toSession(thread: CodexThread): Session {
    return {
      id: thread.id,
      title: thread.title || "Untitled session",
      cwd: thread.cwd,
      updatedAt: thread.updatedAtMs,
      updatedAtLabel: formatDate(thread.updatedAtMs),
      preview: thread.preview || "No preview available.",
      model: thread.model,
      archived: thread.archived,
      rolloutPath: thread.rolloutPath,
    };
  }

  function normalizeWorkspacePath(path: string) {
    const trimmed = path.trim();

    if (!trimmed) {
      return "Unknown workspace";
    }

    const normalized = trimmed.replace(/\\/g, "/");

    if (normalized === "/" || /^[A-Za-z]:\/?$/.test(normalized)) {
      return normalized;
    }

    return normalized.replace(/\/+$/, "");
  }

  function workspaceName(path: string) {
    const normalized = normalizeWorkspacePath(path);
    const parts = normalized.split("/").filter(Boolean);

    return parts.at(-1) ?? normalized;
  }

  function isSameOrChildPath(path: string, root: string) {
    const normalizedPath = normalizeWorkspacePath(path).toLowerCase();
    const normalizedRoot = normalizeWorkspacePath(root).toLowerCase();

    return normalizedPath === normalizedRoot || normalizedPath.startsWith(`${normalizedRoot}/`);
  }

  function classifyWorkspace(path: string): WorkspaceSource {
    if (codexHome?.path && isSameOrChildPath(path, `${codexHome.path}/worktrees`)) {
      return "codexWorktree";
    }

    if (normalizeWorkspacePath(path).includes("/Documents/Codex/")) {
      return "codexTaskFolder";
    }

    return "userProject";
  }

  function workspaceSourceLabel(source: WorkspaceSource) {
    if (source === "codexTaskFolder") {
      return "Codex Task Folder";
    }

    if (source === "codexWorktree") {
      return "Codex Worktree";
    }

    return "User Project";
  }

  function workspaceSourceDescription(source: WorkspaceSource) {
    if (source === "codexTaskFolder") {
      return "Created by the desktop app as a task working directory.";
    }

    if (source === "codexWorktree") {
      return "Managed under the Codex home worktrees directory.";
    }

    return "A project or folder selected as a Codex working directory.";
  }

  function formatSize(bytes: number | null | undefined) {
    if (bytes === null || bytes === undefined) {
      return "Unknown";
    }

    const units = ["B", "KB", "MB", "GB", "TB"];
    let value = bytes;
    let unitIndex = 0;

    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex += 1;
    }

    const digits = value >= 10 || unitIndex === 0 ? 0 : 1;

    return `${value.toFixed(digits)} ${units[unitIndex]}`;
  }

  function formatCount(value: number | null | undefined) {
    if (value === null || value === undefined) {
      return "Unknown";
    }

    return new Intl.NumberFormat().format(value);
  }

  async function loadSessions() {
    isLoading = true;
    loadError = "";

    try {
      codexHome = await invoke<CodexHomeStatus>("get_codex_home_status");
      const threads = await invoke<CodexThread[]>("list_codex_threads");
      sessions = threads.map(toSession);
      selectedId = sessions[0]?.id ?? "";
      selectedWorkspaceId = "";
      workspaceMetadataByPath = {};
      workspaceMetadataErrorsByPath = {};
      loadingWorkspacePath = "";
    } catch (error) {
      loadError = error instanceof Error ? error.message : String(error);
      sessions = [];
      selectedId = "";
      selectedWorkspaceId = "";
      workspaceMetadataByPath = {};
      workspaceMetadataErrorsByPath = {};
      loadingWorkspacePath = "";
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    loadSessions();
  });

  const filteredSessions = $derived(
    sessions.filter((session) => {
      const value = `${session.title} ${session.cwd} ${session.preview} ${session.id}`.toLowerCase();
      return value.includes(query.trim().toLowerCase());
    }),
  );

  const workspaces = $derived.by((): Workspace[] => {
    const workspaceMap = new Map<string, Workspace>();

    for (const session of sessions) {
      const id = normalizeWorkspacePath(session.cwd);
      const existing = workspaceMap.get(id);

      if (existing) {
        existing.sessions.push(session);
        existing.sessionCount += 1;
        existing.activeCount += session.archived ? 0 : 1;
        existing.archivedCount += session.archived ? 1 : 0;

        if (session.updatedAt > existing.updatedAt) {
          existing.updatedAt = session.updatedAt;
          existing.updatedAtLabel = session.updatedAtLabel;
        }

        continue;
      }

      const source = classifyWorkspace(session.cwd);

      workspaceMap.set(id, {
        id,
        name: workspaceName(session.cwd),
        path: session.cwd,
        source,
        sourceLabel: workspaceSourceLabel(source),
        sessions: [session],
        sessionCount: 1,
        activeCount: session.archived ? 0 : 1,
        archivedCount: session.archived ? 1 : 0,
        updatedAt: session.updatedAt,
        updatedAtLabel: session.updatedAtLabel,
      });
    }

    return Array.from(workspaceMap.values()).sort(
      (left, right) => right.updatedAt - left.updatedAt || left.path.localeCompare(right.path),
    );
  });

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
    filteredSessions.find((session) => session.id === selectedId) ?? filteredSessions[0] ?? null,
  );

  const selectedWorkspace = $derived(
    filteredWorkspaces.find((workspace) => workspace.id === selectedWorkspaceId) ??
      filteredWorkspaces[0] ??
      null,
  );

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

    invoke<WorkspaceMetadata>("get_workspace_metadata", { path: selectedWorkspace.path })
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
      <button class="nav-item" type="button">
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
        <strong>Read-first design</strong>
        <span>Indexes and settings stay in one app data directory.</span>
      </div>
    </section>
  </aside>

  <section class="session-list" aria-label="Codex sessions">
    <header class="panel-header">
      <div>
        <p class="eyebrow">Local Codex store</p>
        <h1>{activeView === "sessions" ? "Sessions" : "Workspaces"}</h1>
        {#if codexHome?.path}
          <p class="home-path" title={codexHome.path}>{codexHome.path}</p>
        {/if}
      </div>
      <button class="icon-button" type="button" aria-label="Open Codex data directory">
        <FolderOpen size={18} />
      </button>
    </header>

    <label class="search-box">
      <Search size={18} />
      <input
        bind:value={query}
        type="search"
        placeholder={activeView === "sessions"
          ? "Search title, path, preview, or id"
          : "Search workspace, path, source, or session"}
      />
    </label>

    <div class="list-meta">
      {#if activeView === "sessions"}
        <span>{filteredSessions.length} sessions</span>
      {:else}
        <span>{filteredWorkspaces.length} workspaces / {filteredWorkspaceSessionCount} sessions</span>
      {/if}
      <span>{isLoading ? "Loading" : "Read only"}</span>
    </div>

    <div class="sessions" aria-live="polite">
      {#if isLoading}
        <div class="state-panel">Reading local Codex history...</div>
      {:else if loadError}
        <div class="state-panel error">{loadError}</div>
      {:else if activeView === "sessions" && filteredSessions.length === 0}
        <div class="state-panel">No sessions match the current search.</div>
      {:else if activeView === "workspaces" && filteredWorkspaces.length === 0}
        <div class="state-panel">No workspaces match the current search.</div>
      {:else if activeView === "sessions"}
        {#each filteredSessions as session}
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
        <div>
          <p class="eyebrow">Selected workspace</p>
          <h2>{selectedWorkspace.name}</h2>
        </div>
        <div class="detail-actions" aria-label="Workspace actions">
          <button class="icon-button" type="button" aria-label="Open workspace folder">
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
                activeView = "sessions";
                selectedId = session.id;
              }}
            >
              <span>{session.title}</span>
              <small>{session.updatedAtLabel}</small>
            </button>
          {/each}
        </div>
      </section>
    {:else if activeView === "sessions" && selectedSession}
      <header class="detail-header">
        <div>
          <p class="eyebrow">Selected session</p>
          <h2>{selectedSession.title}</h2>
        </div>
        <div class="detail-actions" aria-label="Session actions">
          <button class="icon-button" type="button" aria-label="Resume session">
            <Play size={17} />
          </button>
          <button class="icon-button" type="button" aria-label="Search transcript">
            <FileSearch size={17} />
          </button>
          <button class="icon-button danger" type="button" aria-label="Move to trash">
            <Trash2 size={17} />
          </button>
        </div>
      </header>

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

      <section class="roadmap">
        <h3>Initial scope</h3>
        <div class="scope-grid">
          <span>Read Codex SQLite thread index</span>
          <span>Group sessions by workspace path</span>
          <span>Inspect workspace metadata lazily</span>
          <span>Parse transcripts on demand</span>
          <span>Keep app metadata isolated</span>
        </div>
      </section>
    {:else}
      <div class="empty-detail">
        Select a {activeView === "sessions" ? "session" : "workspace"} to inspect its local
        metadata.
      </div>
    {/if}
  </section>
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    min-width: 960px;
    color: #1a1f24;
    background: #f5f7f8;
    font-family:
      Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    letter-spacing: 0;
  }

  :global(button),
  :global(input) {
    font: inherit;
  }

  .app-shell {
    display: grid;
    grid-template-columns: 248px minmax(320px, 420px) minmax(480px, 1fr);
    min-height: 100vh;
    background: #f5f7f8;
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 26px;
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
  .roadmap {
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
  .roadmap {
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

  .scope-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .scope-grid span {
    padding: 10px 11px;
    border: 1px solid #dce4e8;
    border-radius: 7px;
    color: #42505b;
    background: #f7fafb;
    font-size: 13px;
    line-height: 19px;
  }
</style>
