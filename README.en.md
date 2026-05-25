# Ferricodex

[中文](README.md) | English

Ferricodex is a Rust-native, local-first desktop manager for Codex history.

The project is intentionally scoped around Codex history rather than a broader agent switcher. The first stable target is a read-first UI for browsing, searching, previewing, and organizing local Codex sessions.

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
- Keep Codex-owned write operations narrow, user-confirmed, and compatible with Codex's own storage layout.

Expected Codex inputs:

```text
~/.codex/state_5.sqlite
~/.codex/session_index.jsonl
~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl
~/.codex/archived_sessions/rollout-*.jsonl
```

Expected app-owned outputs:

```text
<platform app data>/Ferricodex/
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

The project can currently read local Codex thread metadata from `state_5.sqlite`,
render searchable active/archive session and workspace views, reveal local
folders, parse/filter a bounded transcript preview only when a session is
selected, run bounded on-demand global transcript search across active,
archived, or all Codex sessions, and move sessions between Codex-compatible
active/archive locations or delete selected sessions and workspace history with
known Codex database and `session_index.jsonl` cleanup after user confirmation.
Codex-generated task folders under `~/Documents/Codex/YYYY-MM-DD/<folder>` can
be moved to the system Trash with the bound session, optionally after saving a
copy under `~/Documents/Ferricodex Saved Workspaces/`. User project workspaces can be
removed from Codex history without touching project files.

Known Bugs:

- [ ] Windows desktop shortcut icon may appear as a blank white document instead of the Tauri app icon; verify the generated `.ico`, installer shortcut metadata, and Windows icon cache before release.

Completed:

- [x] Scaffold Tauri 2 + Svelte 5 desktop app.
- [x] Install Rust toolchain with a pinned `rust-toolchain.toml`.
- [x] Detect Codex home from `CODEX_HOME` or the default user directory.
- [x] Read the `threads` table from `state_5.sqlite` without modifying Codex files.
- [x] Show active and archived sessions with search, preview, and selected-session details.
- [x] Add a `Sessions` / `Workspaces` split view.
- [x] Group sessions by normalized `cwd` so one workspace can show many related conversations.
- [x] Classify workspace sources, including user projects, generated `~/Documents/Codex/YYYY-MM-DD/<folder>` task folders, and `$CODEX_HOME/worktrees`.
- [x] Show workspace metadata lazily: existence, size, last modified time, and related session count.
- [x] Split frontend API, formatting, opener, and workspace helpers out of the main Svelte page.
- [x] Split Codex backend access into focused `home`, `threads`, `transcript`, and `workspaces` modules.
- [x] Wire folder buttons through a bounded backend opener command for user-home directories.
- [x] Parse transcript JSONL on demand for the selected session with bounded read-only loading.
- [x] Constrain transcript and workspace metadata reads to paths referenced by current Codex history.
- [x] Add per-file and per-line byte limits to transcript JSONL parsing.
- [x] Filter the loaded transcript preview by text and message role without reading extra files.
- [x] Search transcript contents globally with bounded on-demand JSONL scanning across active, archived, or all sessions.
- [x] Archive and unarchive selected sessions by moving rollout JSONL files between `sessions` and `archived_sessions` and updating Codex `state_5.sqlite`.
- [x] Move selected sessions to the system Trash by trashing the rollout JSONL, removing the matching Codex thread row, clearing known thread references, removing the matching `session_index.jsonl` entry, and requiring confirmation.
- [x] Select visible or individual sessions and bulk-delete selected sessions through the same confirmed Codex-compatible Trash cleanup path.
- [x] For generated `~/Documents/Codex/YYYY-MM-DD/<folder>` workspaces, delete the bound session together with the generated folder, or save the folder first and then move the original to Trash.
- [x] Remove user-project workspaces from Codex history by deleting all attached sessions while leaving project files untouched.
- [x] Prepare unsigned multi-platform GitHub release workflow for macOS, Windows, and Linux bundles.
- [x] Validate release tag format and package/Tauri/Cargo version consistency in the release workflow.
- [x] Generate Tauri desktop app icons from the project logo.
- [x] Keep generated build outputs and dependency folders out of Git.
- [x] Push the initial repository to GitHub.

Next:

- [ ] Improve destructive-action recovery messaging when a mutation succeeds but the follow-up refresh fails.
- [ ] Investigate official Codex Desktop deep links before adding any resume launcher.
