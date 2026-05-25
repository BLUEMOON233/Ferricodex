<script lang="ts">
  import { onMount, untrack } from "svelte";
  import {
    getCodexHomeStatus,
    getCodexTranscript,
    getWorkspaceMetadata,
    listCodexThreads,
    moveGeneratedWorkspaceSessionToTrash,
    moveThreadToTrash,
    moveThreadsToTrash,
    searchCodexHistory,
    setThreadArchiveState,
    toSession,
    type CodexHomeStatus,
    type CodexSearchResponse,
    type CodexSearchResult,
    type CodexSearchScope,
    type CodexTranscript,
    type Session,
    type WorkspaceMetadata,
  } from "$lib/codex";
  import { formatCount } from "$lib/formatting";
  import {
    groupSessionsByWorkspace,
    normalizeWorkspacePath,
    type Workspace,
  } from "$lib/workspace";
  import { openLocalPath } from "$lib/opener";
  import { initTheme } from "$lib/theme.svelte";
  import { initZoom, resetZoom, zoomIn, zoomOut } from "$lib/zoom.svelte";
  import { initI18n, t } from "$lib/i18n.svelte";
  import { collapse, initCollapse } from "$lib/collapse.svelte";
  import { toast } from "$lib/toast.svelte";
  import { registerShortcuts } from "$lib/keyboard";
  import { Button, Dialog, Toaster } from "$lib/components/ui";
  import Sidebar from "$lib/components/layout/Sidebar.svelte";
  import { HistoryPanel } from "$lib/components/history";
  import { DetailPanel } from "$lib/components/detail";

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
  let trashActionKey = $state("");
  let pendingTrashSessionId = $state("");
  let pendingTrashWorkspaceId = $state("");
  let pendingBulkTrashSessionIds = $state<string[]>([]);
  let selectedSessionIds = $state<string[]>([]);
  let globalSearchQuery = $state("");
  let globalSearchScope = $state<CodexSearchScope>("active");
  let globalSearchResponse = $state<CodexSearchResponse | null>(null);
  let globalSearchError = $state("");
  let globalSearchLoading = $state(false);
  let pendingTranscriptQuery = $state("");
  let transcriptQuery = $state("");
  let transcriptRoleFilter = $state("all");
  let lastTranscriptPath = "";

  let historyPanel: HistoryPanel | undefined = $state();

  const BULK_KEY = "bulk:sessions";

  function workspaceKey(workspace: Workspace) {
    return `workspace:${workspace.id}`;
  }

  function errorMessage(error: unknown) {
    return error instanceof Error ? error.message : String(error);
  }

  function resetTranscriptFilters(nextQuery = "") {
    transcriptQuery = nextQuery;
    transcriptRoleFilter = "all";
  }

  function resetGlobalSearchResults() {
    globalSearchResponse = null;
    globalSearchError = "";
    globalSearchLoading = false;
  }

  function clearPendingArchive() {
    pendingArchiveSessionId = "";
    pendingArchiveNextArchived = null;
  }

  function clearPendingTrash() {
    pendingTrashSessionId = "";
    pendingTrashWorkspaceId = "";
    pendingBulkTrashSessionIds = [];
  }

  function clearBulkSelection() {
    selectedSessionIds = [];
    pendingBulkTrashSessionIds = [];
  }

  function selectFilter(filter: HistoryFilter) {
    historyFilter = filter;
    selectedKind = "session";
    selectedId = "";
    if (filter !== "settings" && globalSearchScope !== "all") {
      globalSearchScope = filter;
    }
    clearBulkSelection();
    openerError = "";
    archiveError = "";
    trashError = "";
    clearPendingArchive();
    clearPendingTrash();
  }

  function sessionMatchesFilter(session: Session) {
    if (historyFilter === "settings") return false;
    return historyFilter === "archived" ? session.archived : !session.archived;
  }

  function sessionMatchesSearch(session: Session, normalizedQuery: string) {
    if (!normalizedQuery) return true;
    const value = `${session.title} ${session.cwd} ${session.preview} ${session.id} ${session.model ?? ""}`;
    return value.toLowerCase().includes(normalizedQuery);
  }

  function workspaceMatchesSearch(workspace: Workspace, normalizedQuery: string) {
    if (!normalizedQuery) return true;
    const value = `${workspace.name} ${workspace.path} ${workspace.sourceLabel}`;
    return value.toLowerCase().includes(normalizedQuery);
  }

  function buildVisibleWorkspace(
    workspace: Workspace,
    normalizedQuery: string,
  ): WorkspaceListItem | null {
    const workspaceMatches = workspaceMatchesSearch(workspace, normalizedQuery);
    const visibleSessions = workspace.sessions
      .filter(sessionMatchesFilter)
      .filter((session) => workspaceMatches || sessionMatchesSearch(session, normalizedQuery))
      .sort(
        (left, right) => right.updatedAt - left.updatedAt || left.title.localeCompare(right.title),
      );

    if (visibleSessions.length === 0) return null;

    return {
      ...workspace,
      visibleSessions,
      visibleActiveCount: visibleSessions.filter((session) => !session.archived).length,
      visibleArchivedCount: visibleSessions.filter((session) => session.archived).length,
    };
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
    } catch (error) {
      loadError = errorMessage(error);
      sessions = [];
      selectedKind = "session";
      selectedId = "";
    } finally {
      workspaceMetadataByPath = {};
      workspaceMetadataErrorsByPath = {};
      transcriptByPath = {};
      transcriptErrorsByPath = {};
      loadingWorkspacePath = "";
      loadingTranscriptPath = "";
      openerError = "";
      archiveError = "";
      archiveActionSessionId = "";
      clearPendingArchive();
      trashError = "";
      trashActionKey = "";
      clearPendingTrash();
      clearBulkSelection();
      pendingTranscriptQuery = "";
      resetTranscriptFilters();
      resetGlobalSearchResults();
      isLoading = false;
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
      transcriptByPath = { ...transcriptByPath, [path]: transcript };
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

  const workspaces = $derived<Workspace[]>(
    groupSessionsByWorkspace(sessions, codexHome?.path),
  );

  const activeSessionCount = $derived(sessions.filter((session) => !session.archived).length);
  const archivedSessionCount = $derived(sessions.filter((session) => session.archived).length);

  const currentTitle = $derived(
    historyFilter === "settings"
      ? t("history.title.settings")
      : historyFilter === "archived"
        ? t("history.title.archived")
        : t("history.title.active"),
  );

  const currentDescription = $derived(
    historyFilter === "settings"
      ? t("history.description.settings")
      : historyFilter === "archived"
        ? t("history.description.archived")
        : t("history.description.active"),
  );

  const workspaceSections = $derived.by((): WorkspaceSection[] => {
    const normalizedQuery = query.trim().toLowerCase();
    const projects: WorkspaceListItem[] = [];
    const conversations: WorkspaceListItem[] = [];

    for (const workspace of workspaces) {
      const item = buildVisibleWorkspace(workspace, normalizedQuery);
      if (!item) continue;

      if (item.source === "codexTaskFolder") conversations.push(item);
      else projects.push(item);
    }

    const sections: WorkspaceSection[] = [
      {
        id: "projects",
        title: t("section.projects.title"),
        subtitle: t("section.projects.subtitle"),
        workspaces: projects,
        sessionCount: projects.reduce((sum, w) => sum + w.visibleSessions.length, 0),
      },
      {
        id: "conversations",
        title: t("section.conversations.title"),
        subtitle: t("section.conversations.subtitle"),
        workspaces: conversations,
        sessionCount: conversations.reduce((sum, w) => sum + w.visibleSessions.length, 0),
      },
    ];

    return sections.filter((section) => section.workspaces.length > 0);
  });

  const visibleWorkspaceCount = $derived(
    workspaceSections.reduce((total, section) => total + section.workspaces.length, 0),
  );

  const visibleSessionCount = $derived(
    workspaceSections.reduce((total, section) => total + section.sessionCount, 0),
  );

  const visibleSessions = $derived.by((): Session[] => {
    const all: Session[] = [];
    for (const section of workspaceSections) {
      for (const workspace of section.workspaces) {
        all.push(...workspace.visibleSessions);
      }
    }
    return all;
  });

  const firstVisibleSession = $derived<Session | null>(visibleSessions[0] ?? null);

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
    if (historyFilter === "settings" || selectedKind !== "session") return null;
    const found = sessions.find((session) => session.id === selectedId);
    if (found && sessionMatchesFilter(found)) return found;
    return firstVisibleSession;
  });

  const selectedSessionWorkspace = $derived(
    selectedSession
      ? workspaces.find((w) => w.id === normalizeWorkspacePath(selectedSession.cwd)) ?? null
      : null,
  );

  const selectedWorkspace = $derived.by((): Workspace | null => {
    if (historyFilter === "settings") return null;
    if (selectedKind === "workspace") {
      return workspaces.find((w) => w.id === selectedId) ?? firstVisibleWorkspace;
    }
    return selectedSessionWorkspace;
  });

  const selectedWorkspaceMetadata = $derived(
    selectedWorkspace ? workspaceMetadataByPath[selectedWorkspace.id] ?? null : null,
  );

  const selectedWorkspaceMetadataError = $derived(
    selectedWorkspace ? workspaceMetadataErrorsByPath[selectedWorkspace.id] ?? "" : "",
  );

  const selectedWorkspaceMetadataLoading = $derived(
    !!selectedWorkspace && loadingWorkspacePath === selectedWorkspace.id,
  );

  const selectedTranscriptPath = $derived(selectedSession?.rolloutPath.trim() ?? "");

  const selectedTranscript = $derived(
    selectedTranscriptPath ? transcriptByPath[selectedTranscriptPath] ?? null : null,
  );

  const selectedTranscriptError = $derived(
    selectedTranscriptPath
      ? transcriptErrorsByPath[selectedTranscriptPath] ?? ""
      : selectedSession
        ? "Transcript path is unavailable."
        : "",
  );

  const selectedTranscriptIsLoading = $derived(
    selectedTranscriptPath !== "" && loadingTranscriptPath === selectedTranscriptPath,
  );

  const transcriptRoleOptions = $derived.by(() => {
    const roles = selectedTranscript?.messages.map((m) => m.role).filter(Boolean) ?? [];
    return [...new Set(roles)].sort((a, b) => a.localeCompare(b));
  });

  const filteredTranscriptMessages = $derived.by(() => {
    const messages = selectedTranscript?.messages ?? [];
    const normalizedQuery = transcriptQuery.trim().toLowerCase();

    return messages.filter((message) => {
      const matchesRole = transcriptRoleFilter === "all" || message.role === transcriptRoleFilter;
      const value =
        `${message.role} ${message.lineNumber} ${message.timestamp ?? ""} ${message.text}`.toLowerCase();
      const matchesQuery = !normalizedQuery || value.includes(normalizedQuery);
      return matchesRole && matchesQuery;
    });
  });

  const transcriptFilterActive = $derived(
    transcriptQuery.trim() !== "" || transcriptRoleFilter !== "all",
  );

  const pendingArchiveSession = $derived(
    pendingArchiveSessionId
      ? sessions.find((s) => s.id === pendingArchiveSessionId) ?? null
      : null,
  );

  const pendingTrashSession = $derived(
    pendingTrashSessionId ? sessions.find((s) => s.id === pendingTrashSessionId) ?? null : null,
  );

  const pendingTrashWorkspace = $derived(
    pendingTrashWorkspaceId
      ? workspaces.find((w) => w.id === pendingTrashWorkspaceId) ?? null
      : null,
  );

  const pendingTrashSessionWorkspace = $derived(
    pendingTrashSession
      ? workspaces.find((w) => w.id === normalizeWorkspacePath(pendingTrashSession.cwd)) ?? null
      : null,
  );

  const pendingTrashSessionIsGenerated = $derived(
    pendingTrashSessionWorkspace?.source === "codexTaskFolder",
  );

  const pendingTrashWorkspacePrimarySession = $derived(
    pendingTrashWorkspace?.sessions[0] ?? null,
  );

  const pendingBulkTrashSessions = $derived.by((): Session[] => {
    const ids = new Set(pendingBulkTrashSessionIds);
    return sessions.filter((session) => ids.has(session.id));
  });

  const pendingBulkGeneratedCount = $derived.by(
    () =>
      pendingBulkTrashSessions.filter(
        (session) =>
          workspaces.find((w) => w.id === normalizeWorkspacePath(session.cwd))?.source ===
          "codexTaskFolder",
      ).length,
  );

  const pendingBulkSummary = $derived.by(() => {
    const titles = pendingBulkTrashSessions.slice(0, 3).map((s) => s.title);
    const remaining = pendingBulkTrashSessions.length - titles.length;
    return remaining > 0
      ? `${titles.join(", ")} and ${formatCount(remaining)} more`
      : titles.join(", ");
  });

  const archiveDialogOpen = $derived(
    pendingArchiveSession !== null && pendingArchiveNextArchived !== null,
  );
  const trashSessionDialogOpen = $derived(pendingTrashSession !== null);
  const trashWorkspaceDialogOpen = $derived(pendingTrashWorkspace !== null);
  const bulkTrashDialogOpen = $derived(pendingBulkTrashSessions.length > 0);

  function handleQueryChange(value: string) {
    query = value;
  }

  function handleSelectSession(session: Session) {
    selectedKind = "session";
    selectedId = session.id;
    openerError = "";
  }

  function handleSelectWorkspace(workspace: Workspace) {
    selectedKind = "workspace";
    selectedId = workspace.id;
    openerError = "";
  }

  function handleToggleSession(session: Session, checked: boolean) {
    clearPendingTrash();
    trashError = "";

    if (checked) {
      if (!selectedSessionIds.includes(session.id)) {
        selectedSessionIds = [...selectedSessionIds, session.id];
      }
      return;
    }
    selectedSessionIds = selectedSessionIds.filter((id) => id !== session.id);
  }

  function handleSelectVisible() {
    clearPendingTrash();
    trashError = "";
    const ids = new Set(selectedSessionIds);
    for (const session of visibleSessions) ids.add(session.id);
    selectedSessionIds = Array.from(ids);
  }

  function handleClearSelection() {
    clearBulkSelection();
    trashError = "";
  }

  function handleBulkTrash() {
    if (selectedSessionIds.length === 0) {
      trashError = "No sessions are selected.";
      return;
    }
    clearPendingArchive();
    pendingTrashSessionId = "";
    pendingTrashWorkspaceId = "";
    pendingBulkTrashSessionIds = [...selectedSessionIds];
    archiveError = "";
    trashError = "";
  }

  async function openPathWithFeedback(path: string | null | undefined, label: string) {
    const trimmed = path?.trim() ?? "";
    if (!trimmed) {
      openerError = `${label} path is unavailable.`;
      toast.error("Could not open", openerError);
      return;
    }

    try {
      openerError = "";
      await openLocalPath(trimmed);
    } catch (error) {
      openerError = `Could not open ${label}: ${errorMessage(error)}`;
      toast.error("Could not open", openerError);
    }
  }

  async function openCodexHomeDirectory() {
    if (!codexHome?.exists) {
      openerError = codexHome?.path
        ? "Codex home directory does not exist on disk."
        : "Codex home directory is unavailable.";
      toast.error("Codex home unavailable", openerError);
      return;
    }
    await openPathWithFeedback(codexHome.path, "Codex home directory");
  }

  async function openSelectedWorkspaceFolder() {
    const workspace = selectedWorkspace ?? selectedSessionWorkspace;
    if (!workspace) {
      openerError = "No workspace is selected.";
      return;
    }
    if (selectedWorkspaceMetadata && !selectedWorkspaceMetadata.exists) {
      openerError = "Selected workspace does not exist on disk.";
      toast.error("Workspace missing", openerError);
      return;
    }
    await openPathWithFeedback(workspace.path, "workspace folder");
  }

  function requestSessionArchive() {
    if (!selectedSession) {
      archiveError = "No session is selected.";
      return;
    }
    clearPendingTrash();
    pendingArchiveSessionId = selectedSession.id;
    pendingArchiveNextArchived = !selectedSession.archived;
    archiveError = "";
    trashError = "";
  }

  async function confirmArchive() {
    if (!pendingArchiveSessionId || pendingArchiveNextArchived === null) return;

    const sessionId = pendingArchiveSessionId;
    const nextArchived = pendingArchiveNextArchived;
    const action = nextArchived ? "archive" : "restore";
    const session = sessions.find((s) => s.id === sessionId);

    archiveActionSessionId = sessionId;
    archiveError = "";

    try {
      await setThreadArchiveState(sessionId, nextArchived);
      await loadSessions();
      historyFilter = nextArchived ? "archived" : "active";
      selectedKind = "session";
      selectedId = sessionId;
      toast.success(
        nextArchived ? t("toast.archived") : t("toast.restored"),
        session?.title,
      );
    } catch (error) {
      archiveError = errorMessage(error);
      toast.error(
        nextArchived ? t("toast.archived") : t("toast.restored"),
        errorMessage(error),
      );
    } finally {
      archiveActionSessionId = "";
      clearPendingArchive();
    }
  }

  function requestSessionTrash() {
    if (!selectedSession) {
      trashError = "No session is selected.";
      return;
    }
    clearPendingArchive();
    pendingTrashSessionId = selectedSession.id;
    pendingTrashWorkspaceId = "";
    pendingBulkTrashSessionIds = [];
    archiveError = "";
    trashError = "";
  }

  function requestWorkspaceTrash() {
    const workspace = selectedWorkspace;
    if (!workspace) {
      trashError = "No workspace is selected.";
      return;
    }
    clearPendingArchive();
    pendingTrashSessionId = "";
    pendingTrashWorkspaceId = workspace.id;
    pendingBulkTrashSessionIds = [];
    archiveError = "";
    trashError = "";
  }

  async function confirmSingleSessionTrash() {
    if (!pendingTrashSessionId) return;
    const sessionId = pendingTrashSessionId;
    const session = sessions.find((s) => s.id === sessionId);

    trashActionKey = sessionId;
    trashError = "";

    try {
      await moveThreadToTrash(sessionId);
      await loadSessions();
      toast.success(t("toast.sessionTrashed"), session?.title);
    } catch (error) {
      trashError = errorMessage(error);
      toast.error(t("toast.sessionTrashed"), errorMessage(error));
    } finally {
      trashActionKey = "";
      clearPendingTrash();
    }
  }

  async function confirmGeneratedTrash(sessionId: string, saveCopy: boolean) {
    const session = sessions.find((s) => s.id === sessionId);
    trashActionKey = sessionId;
    trashError = "";

    try {
      await moveGeneratedWorkspaceSessionToTrash(sessionId, saveCopy);
      await loadSessions();
      toast.success(
        saveCopy ? t("toast.workspaceTrashedSaved") : t("toast.workspaceTrashedDeleted"),
        session?.title,
      );
    } catch (error) {
      trashError = errorMessage(error);
      toast.error(t("toast.sessionTrashed"), errorMessage(error));
    } finally {
      trashActionKey = "";
      clearPendingTrash();
    }
  }

  async function confirmBulkTrash() {
    const ids = pendingBulkTrashSessions.map((s) => s.id);
    if (ids.length === 0) return;

    trashActionKey = BULK_KEY;
    trashError = "";

    try {
      await moveThreadsToTrash(ids);
      selectedSessionIds = selectedSessionIds.filter((id) => !ids.includes(id));
      clearPendingTrash();
      await loadSessions();
      toast.success(t("toast.bulkTrashed", { count: ids.length }));
    } catch (error) {
      trashError = errorMessage(error);
      toast.error(t("bulk.moveToTrash"), errorMessage(error));
    } finally {
      if (trashActionKey === BULK_KEY) trashActionKey = "";
    }
  }

  async function confirmWorkspaceTrash() {
    if (!pendingTrashWorkspace) return;
    const workspace = pendingTrashWorkspace;
    const ids = workspace.sessions.map((s) => s.id);
    if (ids.length === 0) {
      trashError = errorMessage(new Error("No sessions in workspace."));
      return;
    }

    const key = workspaceKey(workspace);
    trashActionKey = key;
    trashError = "";

    try {
      await moveThreadsToTrash(ids);
      await loadSessions();
      toast.success(t("toast.workspaceTrashedKept"), workspace.name);
    } catch (error) {
      trashError = errorMessage(error);
      toast.error(t("action.moveToTrash"), errorMessage(error));
    } finally {
      if (trashActionKey === key) trashActionKey = "";
      clearPendingTrash();
    }
  }

  async function confirmSessionWorkspaceTrash() {
    if (!pendingTrashSessionWorkspace) {
      trashError = errorMessage(new Error("No workspace."));
      return;
    }
    const workspace = pendingTrashSessionWorkspace;
    const ids = workspace.sessions.map((s) => s.id);
    if (ids.length === 0) {
      trashError = errorMessage(new Error("No sessions in workspace."));
      return;
    }

    const key = workspaceKey(workspace);
    trashActionKey = key;
    trashError = "";

    try {
      await moveThreadsToTrash(ids);
      await loadSessions();
      toast.success(t("toast.workspaceTrashedKept"), workspace.name);
    } catch (error) {
      trashError = errorMessage(error);
      toast.error(t("action.moveToTrash"), errorMessage(error));
    } finally {
      if (trashActionKey === key) trashActionKey = "";
      clearPendingTrash();
    }
  }

  async function runGlobalSearch() {
    const trimmed = globalSearchQuery.trim();
    if (!trimmed) {
      globalSearchResponse = null;
      globalSearchError = "Enter text to search across transcript JSONL files.";
      return;
    }

    globalSearchLoading = true;
    globalSearchError = "";

    try {
      globalSearchResponse = await searchCodexHistory({
        query: trimmed,
        scope: globalSearchScope,
        maxResults: 40,
      });
    } catch (error) {
      globalSearchResponse = null;
      globalSearchError = errorMessage(error);
    } finally {
      globalSearchLoading = false;
    }
  }

  function openGlobalSearchResult(result: CodexSearchResult) {
    const session = sessions.find((s) => s.id === result.threadId);
    if (!session) {
      globalSearchError = "This search result is no longer available. Refresh and search again.";
      return;
    }

    const nextQuery = globalSearchResponse?.query ?? globalSearchQuery.trim();
    const nextPath = session.rolloutPath.trim();

    if (nextPath && nextPath !== selectedTranscriptPath) {
      pendingTranscriptQuery = nextQuery;
    } else {
      resetTranscriptFilters(nextQuery);
    }

    historyFilter = session.archived ? "archived" : "active";
    selectedKind = "session";
    selectedId = session.id;
    openerError = "";
  }

  function navigateSessions(direction: 1 | -1) {
    if (visibleSessions.length === 0) return;

    if (selectedKind !== "session" || !selectedSession) {
      const target = direction === 1 ? visibleSessions[0] : visibleSessions[visibleSessions.length - 1];
      handleSelectSession(target);
      return;
    }

    const currentIndex = visibleSessions.findIndex((s) => s.id === selectedSession.id);
    if (currentIndex === -1) {
      handleSelectSession(visibleSessions[0]);
      return;
    }

    const nextIndex = Math.min(
      Math.max(currentIndex + direction, 0),
      visibleSessions.length - 1,
    );
    if (nextIndex !== currentIndex) {
      handleSelectSession(visibleSessions[nextIndex]);
    }
  }

  $effect(() => {
    const path = selectedTranscriptPath;
    if (path === lastTranscriptPath) return;
    lastTranscriptPath = path;
    resetTranscriptFilters(pendingTranscriptQuery);
    pendingTranscriptQuery = "";
  });

  $effect(() => {
    if (!selectedWorkspace) return;
    const id = selectedWorkspace.id;
    if (workspaceMetadataByPath[id] || loadingWorkspacePath === id) return;

    loadingWorkspacePath = id;
    const { [id]: _err, ...rest } = workspaceMetadataErrorsByPath;
    workspaceMetadataErrorsByPath = rest;

    getWorkspaceMetadata(selectedWorkspace.path)
      .then((metadata) => {
        workspaceMetadataByPath = { ...workspaceMetadataByPath, [id]: metadata };
      })
      .catch((error) => {
        workspaceMetadataErrorsByPath = {
          ...workspaceMetadataErrorsByPath,
          [id]: errorMessage(error),
        };
      })
      .finally(() => {
        if (loadingWorkspacePath === id) loadingWorkspacePath = "";
      });
  });

  $effect(() => {
    if (!selectedSession) return;
    loadTranscriptForSession(selectedSession);
  });

  let lastAutoExpandedSessionId = "";
  $effect(() => {
    if (!selectedSession) return;
    const sessionId = selectedSession.id;
    if (sessionId === lastAutoExpandedSessionId) return;
    const workspaceId = normalizeWorkspacePath(selectedSession.cwd);
    const sectionId =
      selectedSessionWorkspace?.source === "codexTaskFolder" ? "conversations" : "projects";
    untrack(() => {
      if (collapse.isWorkspaceCollapsed(workspaceId)) {
        collapse.expandWorkspace(workspaceId);
      }
      if (collapse.isSectionCollapsed(sectionId)) {
        collapse.expandSection(sectionId);
      }
      lastAutoExpandedSessionId = sessionId;
    });
  });

  onMount(() => {
    initTheme();
    initZoom();
    initI18n();
    initCollapse();
    loadSessions();

    const dispose = registerShortcuts(
      [
        {
          key: "f",
          meta: true,
          handler: () => {
            if (historyFilter === "settings") historyFilter = "active";
            historyPanel?.focusSearch();
          },
        },
        { key: "=", meta: true, handler: zoomIn },
        { key: "+", meta: true, shift: true, handler: zoomIn },
        { key: "-", meta: true, handler: zoomOut },
        { key: "0", meta: true, handler: resetZoom },
        {
          key: "Escape",
          handler: () => {
            if (archiveDialogOpen) {
              clearPendingArchive();
              return;
            }
            if (trashSessionDialogOpen || trashWorkspaceDialogOpen || bulkTrashDialogOpen) {
              clearPendingTrash();
              return;
            }
            if (query) {
              query = "";
              return;
            }
            if (selectedSessionIds.length > 0) {
              clearBulkSelection();
            }
          },
        },
        {
          key: "ArrowDown",
          handler: () => navigateSessions(1),
        },
        {
          key: "ArrowUp",
          handler: () => navigateSessions(-1),
        },
      ],
      {
        allowInEditable: (shortcut) =>
          shortcut.meta === true && ["=", "+", "-", "0"].includes(shortcut.key),
      },
    );

    return dispose;
  });

  const archiveBusy = $derived(
    !!selectedSession && archiveActionSessionId === selectedSession.id,
  );
  const sessionTrashBusy = $derived(
    !!selectedSession && trashActionKey === selectedSession.id,
  );
  const workspaceTrashBusyForSelection = $derived(
    !!selectedWorkspace &&
      (trashActionKey === workspaceKey(selectedWorkspace) ||
        trashActionKey === selectedWorkspace.sessions[0]?.id),
  );
  const bulkBusy = $derived(trashActionKey === BULK_KEY);

  const archiveTitle = $derived(
    pendingArchiveNextArchived ? t("dialog.archive.title") : t("dialog.restore.title"),
  );
  const archiveDescription = $derived(
    pendingArchiveSession
      ? pendingArchiveNextArchived
        ? t("dialog.archive.body", { title: pendingArchiveSession.title })
        : t("dialog.restore.body", { title: pendingArchiveSession.title })
      : "",
  );
</script>

<svelte:head>
  <title>{t("sidebar.brand")}</title>
</svelte:head>

<main class="app-shell" class:settings-mode={historyFilter === "settings"}>
  <Sidebar
    filter={historyFilter}
    activeCount={activeSessionCount}
    archivedCount={archivedSessionCount}
    onSelect={selectFilter}
  />

  {#if historyFilter === "settings"}
    <DetailPanel
      filter={historyFilter}
      selectionKind={selectedKind}
      selectedSession={null}
      selectedWorkspace={null}
      sessionWorkspace={null}
      workspaceMetadata={null}
      workspaceMetadataError=""
      workspaceMetadataLoading={false}
      transcript={null}
      transcriptError=""
      transcriptLoading={false}
      transcriptQuery=""
      transcriptRoleFilter="all"
      transcriptRoleOptions={[]}
      filteredTranscriptMessages={[]}
      transcriptFilterActive={false}
      archiveBusy={false}
      sessionTrashBusy={false}
      workspaceTrashBusy={false}
      archiveError=""
      trashError=""
      {codexHome}
      onOpenWorkspaceFolder={() => {}}
      onToggleArchive={() => {}}
      onTrashSession={() => {}}
      onTrashWorkspace={() => {}}
      onOpenCodexHome={openCodexHomeDirectory}
      onSelectSession={handleSelectSession}
      onTranscriptQueryChange={(v) => (transcriptQuery = v)}
      onTranscriptRoleChange={(v) => (transcriptRoleFilter = v)}
      onClearTranscriptFilters={() => resetTranscriptFilters()}
    />
  {:else}
    <HistoryPanel
      bind:this={historyPanel}
      filter={historyFilter}
      title={currentTitle}
      description={currentDescription}
      {codexHome}
      {isLoading}
      {loadError}
      {openerError}
      sections={workspaceSections}
      {visibleWorkspaceCount}
      {visibleSessionCount}
      selectionKind={selectedKind}
      selectedSessionId={selectedSession?.id ?? ""}
      selectedWorkspaceId={selectedWorkspace?.id ?? ""}
      {selectedSessionIds}
      {visibleSessions}
      {bulkBusy}
      {query}
      {globalSearchQuery}
      {globalSearchScope}
      {globalSearchResponse}
      {globalSearchError}
      {globalSearchLoading}
      onQueryChange={handleQueryChange}
      onSelectWorkspace={handleSelectWorkspace}
      onSelectSession={handleSelectSession}
      onToggleSession={handleToggleSession}
      onSelectVisible={handleSelectVisible}
      onClearSelection={handleClearSelection}
      onBulkTrash={handleBulkTrash}
      onOpenCodexHome={openCodexHomeDirectory}
      onGlobalSearchQueryChange={(v) => (globalSearchQuery = v)}
      onGlobalSearchScopeChange={(v) => (globalSearchScope = v)}
      onGlobalSearchSubmit={runGlobalSearch}
      onGlobalSearchClear={() => {
        globalSearchQuery = "";
        resetGlobalSearchResults();
      }}
      onGlobalSearchSelectResult={openGlobalSearchResult}
    />

    <DetailPanel
      filter={historyFilter}
      selectionKind={selectedKind}
      {selectedSession}
      {selectedWorkspace}
      sessionWorkspace={selectedSessionWorkspace}
      workspaceMetadata={selectedWorkspaceMetadata}
      workspaceMetadataError={selectedWorkspaceMetadataError}
      workspaceMetadataLoading={selectedWorkspaceMetadataLoading}
      transcript={selectedTranscript}
      transcriptError={selectedTranscriptError}
      transcriptLoading={selectedTranscriptIsLoading}
      {transcriptQuery}
      {transcriptRoleFilter}
      {transcriptRoleOptions}
      {filteredTranscriptMessages}
      transcriptFilterActive={transcriptFilterActive}
      archiveBusy={archiveBusy}
      sessionTrashBusy={sessionTrashBusy}
      workspaceTrashBusy={workspaceTrashBusyForSelection}
      {archiveError}
      {trashError}
      {codexHome}
      onOpenWorkspaceFolder={openSelectedWorkspaceFolder}
      onToggleArchive={requestSessionArchive}
      onTrashSession={requestSessionTrash}
      onTrashWorkspace={requestWorkspaceTrash}
      onOpenCodexHome={openCodexHomeDirectory}
      onSelectSession={handleSelectSession}
      onTranscriptQueryChange={(v) => (transcriptQuery = v)}
      onTranscriptRoleChange={(v) => (transcriptRoleFilter = v)}
      onClearTranscriptFilters={() => resetTranscriptFilters()}
    />
  {/if}
</main>

<Dialog
  open={archiveDialogOpen}
  title={archiveTitle}
  description={archiveDescription}
  onclose={clearPendingArchive}
>
  {#if pendingArchiveSession}
    <p class="dialog-target" title={pendingArchiveSession.title}>
      {pendingArchiveSession.title}
    </p>
  {/if}
  {#snippet actions()}
    <Button variant="secondary" onclick={clearPendingArchive}>{t("common.cancel")}</Button>
    <Button
      variant="primary"
      loading={!!pendingArchiveSession && archiveActionSessionId === pendingArchiveSession.id}
      onclick={confirmArchive}
    >
      {pendingArchiveNextArchived ? t("dialog.archive.confirm") : t("dialog.restore.confirm")}
    </Button>
  {/snippet}
</Dialog>

<Dialog
  open={trashSessionDialogOpen}
  title={pendingTrashSessionIsGenerated
    ? t("dialog.trashWorkspace.title")
    : t("dialog.trashSession.title")}
  description={pendingTrashSessionIsGenerated
    ? t("dialog.trashWorkspace.body")
    : pendingTrashSession
      ? t("dialog.trashSession.body", { title: pendingTrashSession.title })
      : ""}
  onclose={clearPendingTrash}
>
  {#if pendingTrashSession}
    <p class="dialog-target" title={pendingTrashSession.title}>{pendingTrashSession.title}</p>
  {/if}
  {#snippet actions()}
    <Button variant="secondary" onclick={clearPendingTrash}>{t("common.cancel")}</Button>
    {#if pendingTrashSession && pendingTrashSessionIsGenerated}
      <Button
        variant="danger"
        loading={trashActionKey === pendingTrashSession.id}
        onclick={() => confirmGeneratedTrash(pendingTrashSession.id, false)}
      >
        {t("dialog.trashWorkspace.deleteOnly")}
      </Button>
      <Button
        variant="danger"
        loading={trashActionKey === pendingTrashSession.id}
        onclick={() => confirmGeneratedTrash(pendingTrashSession.id, true)}
      >
        {t("dialog.trashWorkspace.saveCopy")}
      </Button>
    {:else if pendingTrashSession}
      <Button
        variant="danger"
        loading={trashActionKey === pendingTrashSession.id}
        onclick={confirmSingleSessionTrash}
      >
        {t("dialog.trashSession.confirm")}
      </Button>
    {/if}
  {/snippet}
</Dialog>

<Dialog
  open={trashWorkspaceDialogOpen}
  title={pendingTrashWorkspace?.source === "codexTaskFolder" && pendingTrashWorkspacePrimarySession
    ? t("dialog.trashWorkspace.title")
    : t("dialog.trashWorkspaceHistory.title")}
  description={pendingTrashWorkspace?.source === "codexTaskFolder" && pendingTrashWorkspacePrimarySession
    ? t("dialog.trashWorkspace.body")
    : pendingTrashWorkspace
      ? t("dialog.trashWorkspaceHistory.body", {
          count: pendingTrashWorkspace.sessions.length,
        })
      : ""}
  onclose={clearPendingTrash}
>
  {#if pendingTrashWorkspace}
    {#if pendingTrashWorkspace.source === "codexTaskFolder"}
      <p class="dialog-target" title={pendingTrashWorkspace.path}>{pendingTrashWorkspace.path}</p>
    {:else}
      <p class="dialog-target" title={pendingTrashWorkspace.name}>{pendingTrashWorkspace.name}</p>
      <p class="dialog-note" title={pendingTrashWorkspace.path}>
        {t("dialog.trashWorkspaceHistory.pathKept", { path: pendingTrashWorkspace.path })}
      </p>
    {/if}
  {/if}
  {#snippet actions()}
    <Button variant="secondary" onclick={clearPendingTrash}>{t("common.cancel")}</Button>
    {#if pendingTrashWorkspace?.source === "codexTaskFolder" && pendingTrashWorkspacePrimarySession}
      <Button
        variant="danger"
        loading={trashActionKey === pendingTrashWorkspacePrimarySession.id}
        onclick={() => confirmGeneratedTrash(pendingTrashWorkspacePrimarySession.id, false)}
      >
        {t("dialog.trashWorkspace.deleteOnly")}
      </Button>
      <Button
        variant="danger"
        loading={trashActionKey === pendingTrashWorkspacePrimarySession.id}
        onclick={() => confirmGeneratedTrash(pendingTrashWorkspacePrimarySession.id, true)}
      >
        {t("dialog.trashWorkspace.saveCopy")}
      </Button>
    {:else if pendingTrashWorkspace}
      <Button
        variant="danger"
        loading={trashActionKey === workspaceKey(pendingTrashWorkspace)}
        onclick={confirmWorkspaceTrash}
      >
        {t("dialog.trashWorkspaceHistory.confirm")}
      </Button>
    {/if}
  {/snippet}
</Dialog>

<Dialog
  open={bulkTrashDialogOpen}
  title={t("dialog.bulkTrash.title", { count: pendingBulkTrashSessions.length })}
  description={t("dialog.bulkTrash.body")}
  onclose={clearPendingTrash}
>
  <p>{t("bulk.selected", { count: pendingBulkTrashSessions.length })}</p>
  {#if pendingBulkSummary}
    <p class="dialog-target" title={pendingBulkSummary}>{pendingBulkSummary}</p>
  {/if}
  {#snippet actions()}
    <Button variant="secondary" onclick={clearPendingTrash}>{t("common.cancel")}</Button>
    <Button variant="danger" loading={bulkBusy} onclick={confirmBulkTrash}>
      {t("dialog.bulkTrash.confirm")}
    </Button>
  {/snippet}
</Dialog>

<Toaster />

<style>
  .app-shell {
    display: grid;
    grid-template-columns: var(--sidebar-w) minmax(0, var(--list-w)) minmax(0, 1fr);
    height: 100%;
    min-height: 0;
    overflow: hidden;
    background: var(--bg-window);

    --sidebar-w: 220px;
    --list-w: 380px;
  }

  .app-shell.settings-mode {
    grid-template-columns: var(--sidebar-w) minmax(0, 1fr);
  }

  @media (max-width: 1180px) {
    .app-shell {
      --sidebar-w: 200px;
      --list-w: 340px;
    }
  }

  @media (max-width: 1040px) {
    .app-shell {
      --sidebar-w: 184px;
      --list-w: 320px;
    }
  }

  :global(.dialog-target) {
    overflow-x: auto;
    overflow-y: hidden;
    margin-top: 4px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    background: var(--bg-input-soft);
    color: var(--fg);
    font-size: 12px;
    font-weight: 500;
    overflow-wrap: anywhere;
    scrollbar-width: thin;
    white-space: nowrap;
  }

  :global(.dialog-note) {
    min-width: 0;
    overflow-wrap: anywhere;
  }
</style>
