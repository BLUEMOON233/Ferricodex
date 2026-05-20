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
  app_state/
    store.rs              app-owned SQLite/settings storage
    index.rs              disposable search index
  platform/
    terminal.rs           cross-platform resume launchers
```

## Write Boundaries

Codex-owned files are the source of truth and should be read-only by default. Any write operation must be narrow, user-confirmed, and backed by a recoverable path.

App-owned files are limited to settings, local tags, disposable indexes, and small bounded logs.

## Compatibility Strategy

Codex local storage is an internal format. The app should check schema shape at startup and degrade gracefully when fields are missing or renamed. The UI should expose the detected Codex home and schema status so failures are clear.
