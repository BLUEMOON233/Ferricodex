# Architecture

Codex History Manager is split into a small Rust backend and a Svelte desktop UI.

## Modules

```text
src/
  routes/                 Svelte UI

src-tauri/
  src/
    lib.rs                Tauri command registration
```

Future Rust modules should keep Codex-owned data access separate from app-owned state:

```text
src-tauri/src/
  codex/
    home.rs               CODEX_HOME and platform path detection
    threads.rs            SQLite thread index reader
    transcript.rs         JSONL transcript parser
    workspaces.rs          cwd grouping and read-only workspace metadata
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
- Cleanup features should be advisory and user-confirmed. The app must not
  automatically delete files from `~/.codex`, `~/Documents/Codex`, or project
  directories.

## Write Boundaries

Codex-owned files are the source of truth and should be read-only by default. Any write operation must be narrow, user-confirmed, and backed by a recoverable path.

App-owned files are limited to settings, local tags, disposable indexes, and small bounded logs.

## Compatibility Strategy

Codex local storage is an internal format. The app should check schema shape at startup and degrade gracefully when fields are missing or renamed. The UI should expose the detected Codex home and schema status so failures are clear.
