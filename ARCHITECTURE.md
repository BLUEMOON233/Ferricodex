# Architecture

Codex History Manager is split into a small Rust backend and a Svelte desktop UI.

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
- Cleanup features should be advisory and user-confirmed. The app must not
  automatically delete files from `~/.codex`, `~/Documents/Codex`, or project
  directories.

## Write Boundaries

Codex-owned files are the source of truth and should be read-only by default.
Any write operation must be narrow, user-confirmed, and backed by a recoverable
path. Session archive/unarchive is the first supported write path and must stay
limited to Codex-compatible rollout moves plus the matching `threads` row update.

App-owned files are limited to settings, local tags, disposable indexes, and small bounded logs.

## Compatibility Strategy

Codex local storage is an internal format. The app should check schema shape at startup and degrade gracefully when fields are missing or renamed. The UI should expose the detected Codex home and schema status so failures are clear.
