# Architecture

Ferricodex is split into a small Rust backend and a Svelte desktop UI.

## Modules

```text
src/
  lib/
    codex.ts              Tauri command wrappers and Codex DTO mapping
    formatting.ts         Date, size, and count formatting helpers
    opener.ts             Frontend wrapper for the bounded backend open command
    workspace.ts          Workspace grouping and source classification
  routes/
    +page.svelte          Session, archive, workspace, and transcript UI

src-tauri/
  src/
    lib.rs                Tauri command registration
                          and bounded local folder opening
    codex/
      archive.rs          Codex-compatible session archive/unarchive transitions
      deletion.rs         Confirmed session and workspace Trash transitions
      error.rs            Shared Codex data access errors
      home.rs             CODEX_HOME and platform path detection
      threads.rs          SQLite thread index reader
      transcript.rs       Bounded JSONL transcript parser
      workspaces.rs       Read-only workspace filesystem metadata
```

Future app-owned and platform modules should remain separate from Codex-owned data access:

```text
src-tauri/src/
  app_state/
    store.rs              app-owned SQLite/settings storage
    index.rs              disposable search index
  platform/
    terminal.rs           cross-platform resume launchers
```

## Workspace Model

Codex conversations are attached to a `cwd`, and multiple conversations can share
the same working directory. The app should model that relationship as:

```text
Workspace 1 -> N Threads
```

Workspace management should remain separate from history management:

- The session list is the primary view for browsing conversation history.
- The workspace view groups sessions by normalized `cwd`.
- Directory size and filesystem metadata should be loaded lazily, not during the
  initial session index read.
- Transcript JSONL files should be parsed only after a session is selected, with
  bounded line and message-size limits so large histories cannot block startup.
- Current-session transcript search and role filtering should operate on the
  already-loaded in-memory message list rather than reading additional files.
- Archive/unarchive should preserve Codex's own storage semantics: move rollout
  JSONL files between `sessions/` and `archived_sessions/`, update
  `threads.archived`, `threads.archived_at`, and `threads.rollout_path`, and
  require user confirmation.
- Cleanup features should be explicit and user-confirmed. Trash is treated as
  deletion from the app's perspective; the app does not provide restore flows.
- Session Move to Trash should move the selected rollout JSONL to the system
  Trash, remove the matching `threads` row from `state_5.sqlite`, clear known
  thread references such as `thread_spawn_edges` and
  `agent_job_items.assigned_thread_id`, rely on Codex foreign-key cascades for
  thread-owned metadata, verify no known references remain, and remove the
  matching `session_index.jsonl` entry with a temporary-file replacement and a
  pre-replace concurrent-change check.
- Generated task workspaces are recognized defensively by the shape
  `~/Documents/Codex/YYYY-MM-DD/<folder>` (or an explicit generated-workspace
  root override). A generated workspace is considered bound to one session: the
  delete flow can move the session and generated folder to Trash, or first save
  a copy under `~/Documents/Ferricodex Saved Workspaces/YYYY-MM-DD/<folder>` and then
  move the original folder to Trash. If the folder is missing, deletion
  downgrades to session-only cleanup; if multiple sessions reference the same
  generated folder, folder deletion is stopped for safety.
- User-project workspace removal deletes all Codex sessions attached to that
  normalized `cwd`, including transcripts, thread rows, known references, and
  `session_index.jsonl` entries. It must not modify the project/workspace files.

## Write Boundaries

Codex-owned files are the source of truth and should be read-only by default.
Any write operation must be narrow, user-confirmed, and backed by a recoverable
path. Session archive/unarchive is the first supported write path and must stay
limited to Codex-compatible rollout moves plus the matching `threads` row update.
Session Move to Trash is limited to rollout JSONL files under `sessions/` or
`archived_sessions/`, the matching thread row cleanup, foreign-key cascaded
thread metadata, `thread_spawn_edges`, `agent_job_items.assigned_thread_id`,
and the matching `session_index.jsonl` entry. Generated task-folder deletion is
limited to `~/Documents/Codex/YYYY-MM-DD/<folder>`-shaped directories after the
one-session safety check, with an optional saved copy. User project workspace
removal must only alter Codex history metadata and transcript files, never the
workspace contents.

App-owned files are limited to settings, local tags, disposable indexes, and small bounded logs.

## Compatibility Strategy

Codex local storage is an internal format. The app should check schema shape at startup and degrade gracefully when fields are missing or renamed. The UI should expose the detected Codex home and schema status so failures are clear.
