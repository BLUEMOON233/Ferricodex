import { invoke } from "@tauri-apps/api/core";

import { formatDate } from "./formatting";

export type Session = {
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

export type WorkspaceMetadata = {
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

export type CodexTranscriptMessage = {
  lineNumber: number;
  timestamp: string | null;
  role: string;
  text: string;
};

export type CodexTranscript = {
  path: string;
  exists: boolean;
  lineCount: number;
  invalidLineCount: number;
  truncated: boolean;
  messages: CodexTranscriptMessage[];
};

export type CodexSearchScope = "active" | "archived" | "all";

export type CodexSearchQuery = {
  query: string;
  scope: CodexSearchScope;
  maxResults?: number;
};

export type CodexSearchMatch = {
  lineNumber: number;
  timestamp: string | null;
  role: string;
  snippet: string;
};

export type CodexSearchResult = {
  threadId: string;
  title: string;
  cwd: string;
  rolloutPath: string;
  archived: boolean;
  updatedAtMs: number;
  transcriptTruncated: boolean;
  matches: CodexSearchMatch[];
};

export type CodexSearchResponse = {
  query: string;
  scope: CodexSearchScope;
  scannedThreadCount: number;
  matchedThreadCount: number;
  resultCount: number;
  truncated: boolean;
  results: CodexSearchResult[];
};

export type CodexThread = {
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

export type CodexHomeStatus = {
  path: string;
  exists: boolean;
  stateDbExists: boolean;
  source: "env" | "default";
};

export type CodexProviderConfig = {
  id: string;
  name: string;
  baseUrl: string;
  wireApi: string;
  envKey: string;
  requiresOpenaiAuth: boolean | null;
};

export type CodexProviderSettings = {
  path: string;
  exists: boolean;
  revision: string;
  model: string;
  modelProvider: string;
  providers: CodexProviderConfig[];
  hasSecretFields: boolean;
};

export type CodexProviderSettingsUpdate = {
  revision: string;
  model: string;
  modelProvider: string;
  providers: CodexProviderConfig[];
};

export type CodexAuthStatus = {
  path: string;
  exists: boolean;
  revision: string;
  hasApiKey: boolean;
};

export type CodexApiKeyUpdate = {
  revision: string;
  apiKey: string;
};

export function toSession(thread: CodexThread): Session {
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

export function getCodexHomeStatus() {
  return invoke<CodexHomeStatus>("get_codex_home_status");
}

export function listCodexThreads() {
  return invoke<CodexThread[]>("list_codex_threads");
}

export function getCodexTranscript(path: string) {
  return invoke<CodexTranscript>("get_codex_transcript", { path });
}

export function searchCodexHistory(query: CodexSearchQuery) {
  return invoke<CodexSearchResponse>("search_codex_history", { query });
}

export function getWorkspaceMetadata(path: string) {
  return invoke<WorkspaceMetadata>("get_workspace_metadata", { path });
}

export function getCodexProviderSettings() {
  return invoke<CodexProviderSettings>("get_codex_provider_settings");
}

export function saveCodexProviderSettings(input: CodexProviderSettingsUpdate) {
  return invoke<CodexProviderSettings>("save_codex_provider_settings", { input });
}

export function getCodexAuthStatus() {
  return invoke<CodexAuthStatus>("get_codex_auth_status");
}

export function updateCodexApiKey(input: CodexApiKeyUpdate) {
  return invoke<CodexAuthStatus>("update_codex_api_key", { input });
}

export function setThreadArchiveState(threadId: string, archived: boolean) {
  return invoke<void>("set_thread_archive_state", { threadId, archived });
}

export function moveThreadToTrash(threadId: string) {
  return invoke<void>("move_thread_to_trash", { threadId });
}

export function moveThreadsToTrash(threadIds: string[]) {
  return invoke<void>("move_threads_to_trash", { threadIds });
}

export function moveGeneratedWorkspaceSessionToTrash(
  threadId: string,
  saveWorkspaceCopy: boolean,
) {
  return invoke<void>("move_generated_workspace_session_to_trash", {
    threadId,
    saveWorkspaceCopy,
  });
}
