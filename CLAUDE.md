# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Game Resource Manager (游戏资源管理器) is a local-first Windows desktop app for indexing, browsing, searching, tagging, and favoriting local game asset folders. It is built with **Tauri 2 + React 18 + TypeScript + Rust + SQLite**.

The app does **not** delete, move, rename, or modify original asset files. It only manages indexes, thumbnails, tags, favorites, collections, and notes.

## Common Commands

All npm commands run from the repository root (`I:\GameResManger`). Rust commands run from `src-tauri/`.

### Development

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite frontend dev server only |
| `npm run tauri dev` | Start full Tauri desktop app in dev mode |
| `npm run build` | TypeScript check + Vite production build (outputs `dist/`) |
| `npx tauri build` | Build production desktop bundle (NSIS installer + portable exe) |

### Testing

| Command | Description |
|---------|-------------|
| `npm test` | Run all Vitest frontend tests |
| `npx vitest run src/components/MyComponent.test.tsx` | Run a single frontend test file |
| `cd src-tauri && cargo test` | Run all Rust tests |
| `cd src-tauri && cargo test search::` | Run a single Rust module's tests |
| `cd src-tauri && cargo check` | Fast Rust compilation check |

### Release

`package.ps1` is the Windows release script. It runs `npm run build`, `npx tauri build`, and collects artifacts into `releases/v{VERSION}-{TIMESTAMP}/`.

### Windows Environment Notes

Rust compilation requires MSVC and Windows SDK. If you see errors about missing `stdarg.h`, `excpt.h`, or `msvcrt.lib`, the issue is local tooling—not business code.

## Architecture

### Frontend-Backend Boundary

The frontend never accesses the filesystem or database directly. All local capabilities go through `src/api/tauri.ts`, which uses `invoke` to call Rust commands defined in `src-tauri/src/commands.rs`.

Key rule: when adding a new cross-layer feature, add the `invoke` wrapper in `src/api/tauri.ts`, the Tauri command handler in `commands.rs`, and the business logic in the appropriate Rust module (usually `db.rs`).

### Frontend State Model

Frontend state is centralized in `src/App.tsx` using plain React `useState`/`useEffect`/`useCallback` (no global state library). `App.tsx` orchestrates:

- `assets` / `gridAssets`: master list and current search/filter result
- `selectedIds`: multi-selection state for batch operations
- `activeFilter` / `selectedFolderId` / `selectedCollectionId`: sidebar filters
- `query` / `scope`: search text and search-domain flags
- `latestJobs`: per-folder scan job polling state

State updates use either **local optimistic update** (favorites, notes) or **full reload** (tags, collections, scanning complete).

### Backend State and Lifecycle

`src-tauri/src/lib.rs` bootstraps the app:

1. Registers `dialog` and `opener` plugins.
2. Resolves the app data directory and initializes `data.sqlite` via `db::connect`, running migrations from `src-tauri/migrations/`.
3. Creates the `thumbnails/` subdirectory.
4. Registers three Tauri State objects:
   - `SqlitePool` — database connection pool
   - `ThumbnailDir` — thumbnail cache path
   - `ScanRuntime` — in-memory scan task tracking (active jobs, cancellation tokens, folder locks)
5. Cleans up stale scan jobs in the background.
6. Registers all command handlers.

### Scan Service

`scan_service.rs` runs background folder scans. It is the most complex backend module:

- `ScanRuntime` prevents concurrent scans on the same folder and tracks cancellation.
- A SQLite partial unique index enforces at most one `running` job per folder.
- `WalkDir` traverses the source directory; `indexer.rs` classifies files and decides whether to generate thumbnails.
- Assets are persisted in **batches of 500** to avoid huge transactions.
- During each batch flush, thumbnails are generated concurrently via `spawn_blocking`, then assets are upserted into SQLite.
- `scan_seen_paths` temporarily records every path seen during the scan. At the end, the service marks assets not in this set as `is_missing = true`.
- Cancelling a scan preserves already-written assets and sets the job status to `cancelled`.
- Thumbnail failures do **not** block indexing; the asset is saved with `thumbnail_status = failed`.

### Search

`search.rs` dynamically composes SQL from `AssetSearchRequest`. It can filter by:

- Query text matched against filename, tags, notes, or path
- Asset type, folder, collection, favorite status, missing status

`LIKE` patterns escape `%`, `_`, and backslashes. Results are capped at 2,000 rows.

### Thumbnails

`thumbnails.rs` generates WebP thumbnails (max 320x320) using the Rust `image` crate. The cache filename is a SHA1 of `source_path + modified_at`, so the thumbnail is automatically invalidated when the file changes.

### Data Model

Core SQLite tables:

- `library_folders` — indexed source roots
- `assets` — indexed files with metadata, thumbnail status, and missing flag
- `tags` / `asset_tags` — many-to-many tagging
- `collections` / `collection_assets` — many-to-many collections
- `scan_jobs` — scan task history and progress
- `scan_seen_paths` — ephemeral set used during scanning to detect missing files
- `scan_settings` — per-app scan configuration

### Key Call Paths

Adding a folder:
`LibrarySidebar` -> `App.handlePickFolder` -> `pickLibraryFolder` -> `commands::pick_library_folder` (Tauri dialog) -> `createLibraryFolderFromPath` -> `db::create_library_folder`

Scanning:
`App.handleScanFolder` -> `startScan` -> `commands::start_scan` -> `db::create_scan_job` -> spawn `scan_service::run_scan_job` -> `WalkDir` + `indexer` + `thumbnails` -> `db` writes

Searching:
`SearchToolbar` / sidebar filter change -> `App.executeSearch` -> `searchAssets` -> `commands::search_assets` -> `search::search_assets` -> SQLite query -> `AssetGrid`

## Safety Boundaries

This app manages user local asset files. Protect them:

- **Never** delete, move, rename, or modify original asset files.
- **Never** write cache, thumbnails, or database files into source directories.
- Thumbnails and SQLite data must live in the Tauri app data directory only.
- Allowed file operations: open file, open containing directory, copy absolute path.
- **Never** execute batch-delete commands (`del /s`, `rd /s`, `rm -rf`, `Remove-Item -Recurse`). Single-file deletion with an explicit literal path is acceptable only when clearly required.

## Verification Requirements

After making changes, verify based on scope:

- Frontend / TypeScript changes: `npm test` and/or `npm run build`
- Rust backend changes: `cd src-tauri && cargo test` and/or `cargo check`
- Tauri command or end-to-end behavior changes: `npm run tauri dev` for manual smoke testing
- Scanning, thumbnails, paths, or file-action changes: follow `tests/smoke/README.md` for manual QA

If a check cannot run due to GUI, dependency, or MSVC/SDK issues, state the reason and list the alternative checks you performed.

## UI and Product Constraints

- The main UI is a **three-column workbench**: left sidebar (filters / folders / collections), center (search + asset grid), right (details / batch operations).
- UI text is in Chinese.
- The MVP scope does **not** include: SVN integration, AI auto-tagging, delete, batch move, batch rename, atlas processing, or full 3D/Spine runtime preview.
