const SECTIONS_KEY = "ferricodex.collapsed-sections";
const WORKSPACES_KEY = "ferricodex.collapsed-workspaces";

const DEFAULT_COLLAPSED_SECTIONS: readonly string[] = ["conversations"];

function safeRead(key: string, fallback: readonly string[]): Set<string> {
  if (typeof window === "undefined") {
    return new Set(fallback);
  }

  try {
    const raw = window.localStorage.getItem(key);
    if (raw === null) {
      return new Set(fallback);
    }
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return new Set();
    }
    return new Set(parsed.filter((value): value is string => typeof value === "string"));
  } catch {
    return new Set(fallback);
  }
}

function safeWrite(key: string, value: Set<string>) {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.setItem(key, JSON.stringify([...value]));
  } catch {
    // Ignore unavailable storage and quota errors.
  }
}

let collapsedSections = $state<Set<string>>(new Set(DEFAULT_COLLAPSED_SECTIONS));
let collapsedWorkspaces = $state<Set<string>>(new Set());
let initialized = false;

export function initCollapse() {
  if (initialized || typeof window === "undefined") {
    return;
  }
  initialized = true;
  collapsedSections = safeRead(SECTIONS_KEY, DEFAULT_COLLAPSED_SECTIONS);
  collapsedWorkspaces = safeRead(WORKSPACES_KEY, []);
}

function toggleIn(set: Set<string>, id: string): Set<string> {
  const next = new Set(set);
  if (next.has(id)) {
    next.delete(id);
  } else {
    next.add(id);
  }
  return next;
}

export const collapse = {
  isSectionCollapsed(id: string): boolean {
    return collapsedSections.has(id);
  },
  isWorkspaceCollapsed(id: string): boolean {
    return collapsedWorkspaces.has(id);
  },
  toggleSection(id: string) {
    collapsedSections = toggleIn(collapsedSections, id);
    safeWrite(SECTIONS_KEY, collapsedSections);
  },
  toggleWorkspace(id: string) {
    collapsedWorkspaces = toggleIn(collapsedWorkspaces, id);
    safeWrite(WORKSPACES_KEY, collapsedWorkspaces);
  },
  expandWorkspace(id: string) {
    if (!collapsedWorkspaces.has(id)) {
      return;
    }
    const next = new Set(collapsedWorkspaces);
    next.delete(id);
    collapsedWorkspaces = next;
    safeWrite(WORKSPACES_KEY, collapsedWorkspaces);
  },
  expandSection(id: string) {
    if (!collapsedSections.has(id)) {
      return;
    }
    const next = new Set(collapsedSections);
    next.delete(id);
    collapsedSections = next;
    safeWrite(SECTIONS_KEY, collapsedSections);
  },
};
