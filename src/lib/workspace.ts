import type { Session } from "./codex";

export type ViewMode = "sessions" | "workspaces" | "archive";
export type WorkspaceSource = "codexTaskFolder" | "codexWorktree" | "userProject";

export type Workspace = {
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

export function normalizeWorkspacePath(path: string) {
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

export function workspaceName(path: string) {
  const normalized = normalizeWorkspacePath(path);
  const parts = normalized.split("/").filter(Boolean);

  return parts.at(-1) ?? normalized;
}

export function isSameOrChildPath(path: string, root: string) {
  const normalizedPath = normalizeWorkspacePath(path).toLowerCase();
  const normalizedRoot = normalizeWorkspacePath(root).toLowerCase();

  return normalizedPath === normalizedRoot || normalizedPath.startsWith(`${normalizedRoot}/`);
}

export function isGeneratedCodexTaskWorkspace(path: string) {
  const normalized = normalizeWorkspacePath(path);
  const marker = "/documents/codex/";
  const markerIndex = normalized.toLowerCase().indexOf(marker);

  if (markerIndex === -1) {
    return false;
  }

  const relativePath = normalized.slice(markerIndex + marker.length);
  const parts = relativePath.split("/").filter(Boolean);

  return parts.length === 2 && /^\d{4}-\d{2}-\d{2}$/.test(parts[0]) && parts[1].trim() !== "";
}

export function classifyWorkspace(path: string, codexHomePath?: string): WorkspaceSource {
  if (codexHomePath && isSameOrChildPath(path, `${codexHomePath}/worktrees`)) {
    return "codexWorktree";
  }

  if (isGeneratedCodexTaskWorkspace(path)) {
    return "codexTaskFolder";
  }

  return "userProject";
}

export function workspaceSourceLabel(source: WorkspaceSource) {
  if (source === "codexTaskFolder") {
    return "Codex Task Folder";
  }

  if (source === "codexWorktree") {
    return "Codex Worktree";
  }

  return "User Project";
}

export function workspaceSourceLabelKey(source: WorkspaceSource) {
  if (source === "codexTaskFolder") {
    return "workspace.source.codexTaskFolder";
  }

  if (source === "codexWorktree") {
    return "workspace.source.codexWorktree";
  }

  return "workspace.source.userProject";
}

export function workspaceSourceDescriptionKey(source: WorkspaceSource) {
  if (source === "codexTaskFolder") {
    return "workspace.sourceDescription.codexTaskFolder";
  }

  if (source === "codexWorktree") {
    return "workspace.sourceDescription.codexWorktree";
  }

  return "workspace.sourceDescription.userProject";
}

export function groupSessionsByWorkspace(sessions: Session[], codexHomePath?: string): Workspace[] {
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

    const source = classifyWorkspace(session.cwd, codexHomePath);

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
}
