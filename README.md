# Game Resource Manager / 游戏资源管理器

Game Resource Manager is a local-first Windows desktop app for organizing game art and asset folders into a searchable, previewable resource library.

游戏资源管理器是一个本地优先的 Windows 桌面游戏素材管理器，用于把多个本地素材文件夹整理成可搜索、可预览、可收藏、可标注的资源库。

The app only manages indexes, thumbnails, tags, collections, notes, and recent actions. It does **not** delete, move, rename, or modify original asset files.

本应用只管理索引、缩略图、标签、集合、备注和最近操作记录。它**不会删除、移动、重命名或修改原始素材文件**。

## Status

- Current version: `0.8.0`
- Platform focus: Windows desktop
- Product stage: local-first MVP with scan stability, thumbnail display, folder management, recent activity, tag efficiency, path variants, and advanced filters
- License: MIT

## Features

- Add local asset folders with a native directory picker.
- Scan and index common game asset formats.
- Generate WebP thumbnails for image assets.
- Keep thumbnails and SQLite data in the Tauri app data directory, outside source asset folders.
- Browse assets in a three-column workbench: library filters, asset grid, and details/batch actions.
- Search by file name, tags, notes, and path.
- Filter by type, folder, collection, favorite state, missing state, file size, dimensions, and modified time.
- Sort by file name, size, modified time, or asset type.
- Mark assets as favorites.
- Apply tags to one or more assets, with recent tag suggestions.
- Create, rename, describe, count, delete, and filter collections; add or remove single and multiple assets without changing source files.
- Edit per-asset notes.
- Track recent open/reveal/copy-path actions.
- Open files, reveal containing folders, and copy absolute paths.
- Optionally derive Godot-style `res://` paths when an explicit project root is provided.
- Show scan progress, support cancellation, and preserve already-indexed assets when a scan is cancelled.
- Mark missing files in the index instead of deleting records or touching source files.

## Supported Asset Types

- Image: `png`, `jpg`, `jpeg`, `webp`, `bmp`, `gif`, `psd`
- Audio: `mp3`, `wav`, `ogg`, `flac`, `aac`, `m4a`
- Video: `mp4`, `mov`, `webm`, `avi`, `mkv`
- Font: `ttf`, `otf`, `woff`, `woff2`
- 3D: `gltf`, `glb`, `obj`, `fbx`, `usd`, `usdz`
- Spine: `skel`, `json`, `atlas`, `spine`
- Other files are indexed as `other` when they pass scan rules.

## Tech Stack

- Tauri 2
- React 18
- TypeScript
- Vite
- Rust
- SQLite via `sqlx`
- Rust `image` crate for thumbnails
- Vitest + Testing Library for frontend tests

## Requirements

- Node.js and npm
- Rust stable toolchain
- Tauri prerequisites for Windows
- MSVC build tools and Windows SDK for Rust native dependencies

On Windows, errors about missing `stdarg.h`, `excpt.h`, or `msvcrt.lib` usually indicate a local Visual Studio or Windows SDK configuration issue rather than an application code issue.

## Quick Start

Install dependencies:

```powershell
npm install
```

Run the full desktop app in development mode:

```powershell
npm run tauri dev
```

Run only the Vite frontend:

```powershell
npm run dev
```

Build the frontend:

```powershell
npm run build
```

Build the Tauri app:

```powershell
npx tauri build
```

Create local release artifacts under `releases/`:

```powershell
.\package.ps1
```

`releases/` is intentionally ignored by Git. Publish release binaries through GitHub Releases instead of committing them to the repository.

## Verification

Run frontend tests:

```powershell
npm test
```

Run Rust tests:

```powershell
cd src-tauri
cargo test
```

Run Rust compile checks:

```powershell
cd src-tauri
cargo check
```

Current local verification baseline:

- `npm test`: 71 frontend tests passing.
- `npm run build`: succeeds.
- `cargo test`: 107 Rust unit tests plus the build script integration test passing.
- `cargo check`: succeeds when the local MSVC and Windows SDK environment is configured.

Manual smoke checks live in [tests/smoke/README.md](tests/smoke/README.md).

## Project Structure

```text
.
|-- src/
|   |-- api/             # Typed Tauri command wrappers
|   |-- components/      # React workbench UI components
|   |-- test/            # Frontend test setup
|   |-- types/           # Frontend domain types
|   |-- App.tsx          # Main UI state and command orchestration
|   |-- main.tsx         # React entry point
|   `-- styles.css       # App styles
|-- src-tauri/
|   |-- capabilities/    # Tauri v2 permissions
|   |-- migrations/      # SQLite schema migrations
|   |-- src/
|   |   |-- commands.rs      # Tauri command boundary
|   |   |-- db.rs            # SQLite connection and repositories
|   |   |-- file_actions.rs  # Safe OS file actions
|   |   |-- indexer.rs       # File classification and scan rules
|   |   |-- lib.rs           # Tauri bootstrap and state registration
|   |   |-- models.rs        # Rust domain models
|   |   |-- scan_service.rs  # Background scan jobs
|   |   |-- search.rs        # Search/filter SQL composition
|   |   |-- tags.rs          # Tag normalization
|   |   `-- thumbnails.rs    # Thumbnail generation
|   |-- Cargo.toml
|   `-- tauri.conf.json
|-- tests/
|   `-- smoke/           # Manual QA checklist
|-- package.json
|-- package.ps1
|-- tsconfig.json
`-- vite.config.ts
```

## Architecture

The frontend never accesses the file system or database directly. It calls typed wrappers in `src/api/tauri.ts`, which invoke Rust commands in `src-tauri/src/commands.rs`.

Rust owns local capabilities:

- folder picking and folder management
- scanning and incremental indexing
- thumbnail generation
- SQLite migrations and repositories
- search/filter query construction
- safe OS actions: open file, reveal folder, copy path

SQLite and thumbnails are stored in the Tauri app data directory. Source asset folders remain untouched.

More details:

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)
- [MODULE_MAP.md](MODULE_MAP.md)

## Safety Boundaries

This project manages user-owned local asset folders. Keep these rules intact:

- Do not delete source asset files.
- Do not move source asset files.
- Do not rename source asset files.
- Do not modify source asset contents.
- Do not write thumbnails, caches, databases, or metadata into source asset folders.
- Missing files are marked as missing in SQLite instead of being removed from disk.
- Removing a library folder removes app index records only; it does not remove the source directory.

## Current Limitations

- The app is focused on Windows desktop.
- Search results are loaded in pages and the grid is virtualized; extremely large libraries may still need additional profiling and query tuning.
- CJK substring search uses SQLite trigram indexing for terms with at least three Unicode characters. One- and two-character terms use a scope-limited literal fallback and may be slower on extremely large libraries.
- Thumbnail generation is decoupled from indexing failures, but image-heavy folders can still make scans take time.
- PSD, Spine, 3D, audio, and video files are indexed and shown with useful placeholders, but the app does not provide full runtime previews for those formats.
- Godot `res://` path derivation is optional and only works when an explicit project root is provided.
- The app does not include cloud sync, team accounts, AI auto-tagging, atlas processing, SVN integration, delete, batch move, or batch rename.

## Contributing

This is an early local-first desktop project. Before changing behavior, preserve the source-file safety boundary and run the relevant checks:

- Frontend or TypeScript changes: `npm test` and `npm run build`
- Rust backend changes: `cargo test` and `cargo check` in `src-tauri`
- Scan, thumbnail, path, or file-action changes: also follow [tests/smoke/README.md](tests/smoke/README.md)

## License

MIT License. See [LICENSE](LICENSE).
