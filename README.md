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

The project can currently read local Codex thread metadata from `state_5.sqlite`
in read-only mode, render searchable session and workspace views, reveal local
folders, and parse a bounded transcript preview only when a session is selected.

Completed:

- [x] Scaffold Tauri 2 + Svelte 5 desktop app.
- [x] Install Rust toolchain with a pinned `rust-toolchain.toml`.
- [x] Detect Codex home from `CODEX_HOME` or the default user directory.
- [x] Read the `threads` table from `state_5.sqlite` without modifying Codex files.
- [x] Show active and archived sessions with search, preview, and selected-session details.
- [x] Add a `Sessions` / `Workspaces` split view.
- [x] Group sessions by normalized `cwd` so one workspace can show many related conversations.
- [x] Classify workspace sources, including user projects, `~/Documents/Codex` task folders, and `$CODEX_HOME/worktrees`.
- [x] Show workspace metadata lazily: existence, size, last modified time, and related session count.
- [x] Split frontend API, formatting, opener, and workspace helpers out of the main Svelte page.
- [x] Split Codex backend access into focused `home`, `threads`, `transcript`, and `workspaces` modules.
- [x] Wire folder buttons through a bounded backend opener command for user-home directories.
- [x] Parse transcript JSONL on demand for the selected session with bounded read-only loading.
- [x] Keep generated build outputs and dependency folders out of Git.
- [x] Push the initial repository to GitHub.

Next:

- [ ] Launch `codex resume <session-id>` through platform-specific terminal helpers.
- [ ] Add UI-level tests or an automation-friendly test route for reliable desktop interaction checks.
- [ ] Add conservative cleanup suggestions for empty or orphaned workspaces, with no automatic deletion.
