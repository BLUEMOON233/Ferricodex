# Codex History Manager

A lightweight, local-first desktop manager for Codex conversation history.

The project is intentionally scoped around Codex history rather than a broader agent switcher. The first stable target is a read-first UI for browsing, searching, previewing, and resuming local Codex sessions.

## Stack

- Tauri 2 for the desktop shell
- Rust for Codex data access, indexing, and safe file operations
- Svelte 5 + SvelteKit static adapter + Vite for the UI
- TypeScript for frontend code
- Tailwind CSS 4 as the styling foundation
- shadcn-svelte/Bits UI planned for reusable controls as the UI grows

## Local Data Policy

The app should avoid environment pollution:

- Do not modify shell profiles, PATH, login items, or system services.
- Read Codex data from `CODEX_HOME` when set, otherwise from the platform user directory.
- Keep app-owned settings, indexes, and logs in one app data directory.
- Treat search indexes as disposable cache that can be rebuilt from Codex session files.
- Prefer read-only behavior for Codex-owned files until write operations are explicitly designed.

Expected Codex inputs:

```text
~/.codex/state_5.sqlite
~/.codex/session_index.jsonl
~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl
```

Expected app-owned outputs:

```text
<platform app data>/Codex History Manager/
  settings.json
  app.db
  search-index/
  logs/
```

## Development

Install prerequisites:

- Node.js 18 or newer
- Rust toolchain with `cargo`
- Tauri desktop prerequisites for your OS

Install frontend dependencies:

```bash
npm install
```

Run frontend checks:

```bash
npm run check
```

Run the desktop app:

```bash
npm run tauri dev
```

Build a release bundle:

```bash
npm run tauri build
```

## Current Status

This repository currently contains the initial Tauri/Svelte project shell and UI scaffold. The backend data reader is intentionally not wired yet.

Planned first implementation slice:

1. Detect Codex home directory.
2. Read the `threads` table from `state_5.sqlite`.
3. Show active and archived sessions.
4. Parse transcript JSONL only when a session is selected.
5. Launch `codex resume <session-id>` through platform-specific terminal helpers.
