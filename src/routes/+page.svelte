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
  let selectedId = $state("");
  let sessions = $state<Session[]>([]);
  let codexHome = $state<CodexHomeStatus | null>(null);
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

  async function loadSessions() {
    isLoading = true;
    loadError = "";

    try {
      codexHome = await invoke<CodexHomeStatus>("get_codex_home_status");
      const threads = await invoke<CodexThread[]>("list_codex_threads");
      sessions = threads.map(toSession);
      selectedId = sessions[0]?.id ?? "";
    } catch (error) {
      loadError = error instanceof Error ? error.message : String(error);
      sessions = [];
      selectedId = "";
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

  const selectedSession = $derived(
    filteredSessions.find((session) => session.id === selectedId) ?? filteredSessions[0] ?? null,
  );
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
      <button class="nav-item active" type="button">
        <Database size={17} />
        Sessions
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
        <h1>Sessions</h1>
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
      <input bind:value={query} type="search" placeholder="Search title, path, preview, or id" />
    </label>

    <div class="list-meta">
      <span>{filteredSessions.length} sessions</span>
      <span>{isLoading ? "Loading" : "Read only"}</span>
    </div>

    <div class="sessions" aria-live="polite">
      {#if isLoading}
        <div class="state-panel">Reading local Codex history...</div>
      {:else if loadError}
        <div class="state-panel error">{loadError}</div>
      {:else if filteredSessions.length === 0}
        <div class="state-panel">No sessions match the current search.</div>
      {:else}
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
      {/if}
    </div>
  </section>

  <section class="detail-panel" aria-label="Session details">
    {#if selectedSession}
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
          <span>Parse JSONL transcripts on demand</span>
          <span>Resume by session id</span>
          <span>Keep app metadata isolated</span>
        </div>
      </section>
    {:else}
      <div class="empty-detail">Select a session to inspect its local metadata.</div>
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

  .session-row:hover,
  .session-row.active {
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
  .roadmap {
    padding: 17px;
  }

  .preview p {
    margin-bottom: 0;
    color: #44515d;
    line-height: 24px;
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
