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

export function getWorkspaceMetadata(path: string) {
  return invoke<WorkspaceMetadata>("get_workspace_metadata", { path });
}
