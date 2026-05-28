# Scan Stability Plus Implementation Plan

**Plan Version:** v0.1
**Last Updated:** 2026-05-28

> For agentic workers: REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (- [ ]) syntax for tracking.

Goal: Make release scans stable, visible, cancellable, incremental, and safe for large local game asset folders such as G:\资源\2D游戏资源_淘宝\.

Architecture: Keep Tauri + React + SQLite. Rust owns scan jobs, batching, cancellation, scan rules, and thumbnail queue state; React starts/cancels scans and polls status. Indexing and thumbnail generation are separated so bad images or slow PSD files never roll back asset indexing.

Tech Stack: Tauri 2, React, TypeScript, Vite, Rust, SQLite via sqlx, walkdir, image, Vitest, cargo test.

---

## Source Context

- Product spec: G:\oklai999的策划仓库\游戏资源管理器\需求整理_v0.1.md
- Original implementation plan: G:\oklai999的策划仓库\游戏资源管理器\实施计划_v0.1.md
- Work directory: I:\GameResManger
- Known release symptom: folder add succeeds, scan leaves assets = 0, scan_jobs empty, and last_scanned_at = null.
- Known large folder profile: about 62,429 supported files, about 62,063 images, about 8 GB image data.

## Safety Constraints

- Do not delete, move, rename, or modify original asset files.
- Do not run batch deletion commands: del /s, rd /s, rmdir /s, Remove-Item -Recurse, or rm -rf.
- Thumbnail cache and SQLite data may be updated; source asset folders must remain untouched.

## File Structure

Modify existing files:

- src-tauri/migrations/0002_scan_jobs_and_thumbnails.sql: add scan job progress fields, thumbnail status fields, and scan settings tables.
- src-tauri/src/models.rs: add ScanJob, ScanJobStatus, ScanProgress, ScanSettings, ThumbnailStatus, and command response models.
- src-tauri/src/indexer.rs: add scan rules, ignored directory checks, incremental decisions, and scanner tests.
- src-tauri/src/db.rs: add batch upsert, scan job lifecycle, seen-path staging, missing marking, scan settings, and thumbnail status repository functions.
- src-tauri/src/thumbnails.rs: keep thumbnail path helpers and add decision helpers that skip PSD thumbnail generation by default.
- src-tauri/src/scan_service.rs: new backend service for batch scanning, progress updates, cancellation checks, and thumbnail enqueue decisions.
- src-tauri/src/thumbnail_queue.rs: new backend service foundation for bounded thumbnail generation after indexing.
- src-tauri/src/commands.rs: expose scan start/cancel/status/settings commands and keep existing commands compatible.
- src-tauri/src/lib.rs: register ScanRuntime and ThumbnailRuntime state and new Tauri commands.
- src/types/asset.ts: add scan job, scan settings, thumbnail status, and command response types.
- src/api/tauri.ts: add typed wrappers for scan start/cancel/status/settings commands.
- src/App.tsx: wire scan state polling and refresh assets after scan progress changes.
- src/components/LibrarySidebar.tsx: show scan/cancel controls and per-folder status.
- src/components/ScanStatusBar.tsx: new status surface for running/completed/failed/cancelled jobs.
- src/components/ScanSettingsPanel.tsx: new lightweight scan rule controls.
- src/components/AssetGrid.tsx: show thumbnail queued/generating/failed placeholders.
- src/styles.css: style scan controls and status surfaces.
- tests/smoke/README.md: add large-folder release smoke checklist.

---

### Task 1: Add Scan State Schema

Files:
- Create: src-tauri/migrations/0002_scan_jobs_and_thumbnails.sql
- Modify: src-tauri/src/models.rs

- [x] Step 1: Create migration with these exact changes:

    ALTER TABLE scan_jobs ADD COLUMN cancelled_at TEXT;
    ALTER TABLE scan_jobs ADD COLUMN unchanged_count INTEGER NOT NULL DEFAULT 0;
    ALTER TABLE scan_jobs ADD COLUMN current_path TEXT;
    ALTER TABLE assets ADD COLUMN thumbnail_status TEXT NOT NULL DEFAULT 'none';
    ALTER TABLE assets ADD COLUMN thumbnail_error TEXT;
    CREATE TABLE IF NOT EXISTS scan_seen_paths (
      scan_job_id INTEGER NOT NULL,
      absolute_path TEXT NOT NULL,
      PRIMARY KEY (scan_job_id, absolute_path),
      FOREIGN KEY (scan_job_id) REFERENCES scan_jobs(id) ON DELETE CASCADE
    );
    CREATE TABLE IF NOT EXISTS scan_settings (
      id INTEGER PRIMARY KEY CHECK (id = 1),
      include_images INTEGER NOT NULL DEFAULT 1,
      include_audio INTEGER NOT NULL DEFAULT 1,
      include_video INTEGER NOT NULL DEFAULT 1,
      include_fonts INTEGER NOT NULL DEFAULT 1,
      include_models INTEGER NOT NULL DEFAULT 1,
      include_spine INTEGER NOT NULL DEFAULT 1,
      include_psd INTEGER NOT NULL DEFAULT 1,
      generate_psd_thumbnails INTEGER NOT NULL DEFAULT 0,
      ignored_directory_names TEXT NOT NULL DEFAULT 'node_modules,.git,.godot,target,dist,build,.codex_spreadsheet_tinyswords',
      updated_at TEXT NOT NULL
    );
    INSERT OR IGNORE INTO scan_settings (id, updated_at) VALUES (1, datetime('now'));
    CREATE INDEX IF NOT EXISTS idx_scan_seen_paths_job ON scan_seen_paths(scan_job_id);
    CREATE INDEX IF NOT EXISTS idx_scan_jobs_folder_status ON scan_jobs(library_folder_id, status);
    CREATE INDEX IF NOT EXISTS idx_assets_thumbnail_status ON assets(thumbnail_status);

- [x] Step 2: Add Rust models: ScanJobStatus, ScanJob, ScanSettings, ScanProgress. Fields must match frontend snake_case JSON: id, library_folder_id, status, started_at, finished_at, cancelled_at, found_count, added_count, updated_count, unchanged_count, missing_count, skipped_count, current_path, error_message.
- [x] Step 3: Run: cd I:\GameResManger\src-tauri; cargo check. Expected: compile succeeds or only reports functions added in later tasks.
- [x] Step 4: Commit: git add src-tauri/migrations/0002_scan_jobs_and_thumbnails.sql src-tauri/src/models.rs; git commit -m "feat: add scan job state schema".

---

### Task 2: Implement Scan Job Repository Functions

Files:
- Modify: src-tauri/src/db.rs

- [x] Step 1: Add create_scan_job(db, folder_id) returning ScanJob. It inserts status running and started_at now.
- [x] Step 2: Add get_scan_job(db, job_id) and latest_scan_job_for_folder(db, folder_id). Both map all ScanJob fields.
- [x] Step 3: Add update_scan_job_progress(db, job_id, found, added, updated, unchanged, skipped, current_path).
- [x] Step 4: Add finish_scan_job(db, job_id, missing), fail_scan_job(db, job_id, message), and cancel_scan_job(db, job_id). finished_at must be set for completed, failed, and cancelled.
- [x] Step 5: Add add_seen_paths(db, job_id, paths). Insert into scan_seen_paths in a transaction, one explicit path per row.
- [x] Step 6: Add mark_missing_assets_from_seen(db, folder_id, job_id). Use subquery against scan_seen_paths, not a giant placeholder list.

    UPDATE assets
    SET is_missing = 1, updated_at = ?1
    WHERE library_folder_id = ?2
      AND absolute_path NOT IN (
        SELECT absolute_path FROM scan_seen_paths WHERE scan_job_id = ?3
      );

- [x] Step 7: Add get_scan_settings(db) and save_scan_settings(db, settings).
- [x] Step 8: Run: cd I:\GameResManger\src-tauri; cargo check. Expected: backend compiles.
- [x] Step 9: Commit: git add src-tauri/src/db.rs; git commit -m "feat: add scan job repository functions".

---

### Task 3: Add Scan Rules and Incremental Decisions

Files:
- Modify: src-tauri/src/indexer.rs

- [x] Step 1: Add should_ignore_dir(path, settings). It compares directory file_name against settings.ignored_directory_names case-insensitively.
- [x] Step 2: Add asset_type_allowed(asset_type, extension, settings). PSD obeys include_psd; other image formats obey include_images.
- [x] Step 3: Add should_generate_thumbnail(extension, settings). png, jpg, jpeg, webp, bmp, gif return true; psd returns settings.generate_psd_thumbnails; all others false.
- [x] Step 4: Add scan_folder_with_settings(path, settings). It must use WalkDir iterator skip_current_dir for ignored directories and must keep old scan_folder for compatibility tests.
- [x] Step 5: Add tests: ignores_configured_directories, psd_thumbnail_generation_is_off_by_default, disabled_image_scan_skips_png.
- [x] Step 6: Run: cd I:\GameResManger\src-tauri; cargo test indexer. Expected: all indexer tests pass.
- [x] Step 7: Commit: git add src-tauri/src/indexer.rs; git commit -m "feat: add scan rules".

---

### Task 4: Create Cancellable Batch Scan Service

Files:
- Create: src-tauri/src/scan_service.rs
- Modify: src-tauri/src/lib.rs

- [x] Step 1: Create ScanRuntime with Arc<Mutex<HashSet<i64>>> and methods cancel(job_id), is_cancelled(job_id), clear(job_id).
- [x] Step 2: Add ScanCounters with found, added, updated, unchanged, skipped.
- [x] Step 3: Add constant SCAN_BATCH_SIZE = 500.
- [x] Step 4: Implement persist_batch(db_pool, folder_id, job_id, batch, counters). It adds seen paths, checks existing asset by absolute_path, increments unchanged when file_size and modified_at match, and upserts changed assets.
- [x] Step 5: Implement run_scan_job(db_pool, runtime, thumbnail_dir, folder_id, job_id). It loads folder and settings, scans with settings, writes every 500 assets, updates progress, checks cancellation between assets, marks missing through scan_seen_paths, finishes job, and records failure through fail_scan_job in the caller.
- [x] Step 6: Register module in lib.rs: mod scan_service; and app.manage(scan_service::ScanRuntime::default()) inside setup.
- [x] Step 7: Run: cd I:\GameResManger\src-tauri; cargo check. Expected: backend compiles.
- [x] Step 8: Commit: git add src-tauri/src/scan_service.rs src-tauri/src/lib.rs; git commit -m "feat: add cancellable batch scan service".

---

### Task 5: Expose Scan Commands

Files:
- Modify: src-tauri/src/commands.rs
- Modify: src-tauri/src/lib.rs
- Modify: src/types/asset.ts
- Modify: src/api/tauri.ts

- [x] Step 1: Add frontend types ScanJobStatus, ScanJob, ScanSettings. Field names must match Rust JSON exactly.
- [x] Step 2: Add API wrappers: startScan(folderId), cancelScan(jobId), latestScanJob(folderId), getScanSettings(), saveScanSettings(settings).
- [x] Step 3: Add Tauri commands start_scan, cancel_scan, latest_scan_job, get_scan_settings, save_scan_settings.
- [x] Step 4: start_scan must create a scan job, spawn run_scan_job, return the job immediately, and fail the job if the spawned task returns an error.
- [x] Step 5: cancel_scan must set runtime cancellation and update database status to cancelled.
- [x] Step 6: Register new commands in lib.rs generate_handler.
- [x] Step 7: Run: cd I:\GameResManger; npm run build; cd src-tauri; cargo check. Expected: frontend and backend compile.
- [x] Step 8: Commit: git add src/types/asset.ts src/api/tauri.ts src-tauri/src/commands.rs src-tauri/src/lib.rs; git commit -m "feat: expose scan job commands".

---

### Task 6: Add Scan Feedback UI

Files:
- Create: src/components/ScanStatusBar.tsx
- Modify: src/components/LibrarySidebar.tsx
- Modify: src/App.tsx
- Modify: src/styles.css

- [x] Step 1: Create ScanStatusBar. It shows status label, found, added, updated, unchanged, missing, skipped, and error_message.
- [x] Step 2: Update LibrarySidebar props to accept latestJobs and onCancelScan. If latestJobs[folder.id].status is running, show Cancel; otherwise show Scan.
- [x] Step 3: Update App state with latestJobs: Record<number, ScanJob | null>.
- [x] Step 4: After loadData loads folderList, call latestScanJob for each folder and store results.
- [x] Step 5: Replace old scanLibraryFolder call with startScan. Do not await scan completion in the click handler.
- [x] Step 6: Add polling every 1000 ms while any latest job is running; polling calls loadData.
- [x] Step 7: Render ScanStatusBar for each latest job above AssetGrid.
- [x] Step 8: Add CSS for scan-status running/completed/failed/cancelled.
- [x] Step 9: Run: cd I:\GameResManger; npm test; npm run build. Expected: tests and build pass.
- [x] Step 10: Commit: git add src/App.tsx src/components/LibrarySidebar.tsx src/components/ScanStatusBar.tsx src/styles.css; git commit -m "feat: show scan progress and cancellation".

---

### Task 7: Add Scan Settings UI

Files:
- Create: src/components/ScanSettingsPanel.tsx
- Modify: src/App.tsx
- Modify: src/styles.css

- [x] Step 1: Create ScanSettingsPanel with checkboxes for images, audio, video, fonts, models, spine, PSD indexing, and PSD thumbnails. Display ignored_directory_names as read-only text in this iteration.
- [x] Step 2: In App, load getScanSettings on mount.
- [x] Step 3: On checkbox change, optimistically update local state and call saveScanSettings.
- [x] Step 4: Render panel in the left sidebar area below folder list.
- [x] Step 5: Run: cd I:\GameResManger; npm run build. Expected: frontend build passes.
- [x] Step 6: Commit: git add src/App.tsx src/components/ScanSettingsPanel.tsx src/styles.css; git commit -m "feat: add scan settings panel".

---

### Task 8: Add Thumbnail Queue Status Foundation

Files:
- Create: src-tauri/src/thumbnail_queue.rs
- Modify: src-tauri/src/lib.rs
- Modify: src-tauri/src/models.rs
- Modify: src-tauri/src/db.rs
- Modify: src/types/asset.ts
- Modify: src/components/AssetGrid.tsx

- [x] Step 1: Create ThumbnailRuntime with running flag. This task creates the state foundation; a full concurrent worker can be a later plan.
- [x] Step 2: Add mark_thumbnail_failed(db, asset_id, message) helper.
- [x] Step 3: Register module and manage ThumbnailRuntime in lib.rs.
- [x] Step 4: Add Asset fields thumbnail_status and thumbnail_error in Rust and TypeScript.
- [x] Step 5: Update db::list_assets SELECT and mapping to include thumbnail_status and thumbnail_error.
- [x] Step 6: Update AssetGrid fallback display: failed shows 缩略图失败, queued/generating shows 生成中, none shows extension.
- [x] Step 7: Run: cd I:\GameResManger; npm test; npm run build; cd src-tauri; cargo test; cargo check. Expected: all checks pass.
- [x] Step 8: Commit: git add src-tauri/src/thumbnail_queue.rs src-tauri/src/lib.rs src-tauri/src/models.rs src-tauri/src/db.rs src/types/asset.ts src/components/AssetGrid.tsx; git commit -m "feat: prepare thumbnail queue status".

---

### Task 9: Update Large-Folder Smoke QA

Files:
- Modify: tests/smoke/README.md

- [x] Step 1: Append Large Folder Release Scan checklist. Include add folder, click scan, confirm Cancel appears within one second, scan_jobs running row, asset count increases, cancel works, second scan shows unchanged_count, ignored dirs are skipped, PSD thumbnails are off by default, and source files are untouched.
- [x] Step 2: Commit: git add tests/smoke/README.md; git commit -m "docs: add large scan smoke checklist".

---

### Task 10: Final Verification

Files:
- No new files.

- [x] Step 1: Run frontend tests: cd I:\GameResManger; npm test. Expected: all pass.
- [x] Step 2: Run frontend build: cd I:\GameResManger; npm run build. Expected: build passes.
- [x] Step 3: Run backend tests: cd I:\GameResManger\src-tauri; cargo test. Expected: all pass.
- [x] Step 4: Run backend check: cd I:\GameResManger\src-tauri; cargo check. Expected: compiles.
- [x] Step 5: Build release: cd I:\GameResManger; npm run tauri build. Expected: release executable and bundle are produced. **Result:** release executable built successfully at `src-tauri/target/release/game-resource-manager.exe`. MSI bundling failed due to Wix `light.exe` environment issue, not code-related.
- [ ] Step 6: Run manual release smoke with G:\资源\2D游戏资源_淘宝\. Expected: scan starts visibly, can be cancelled, can complete or fail with visible error, inserts assets, second scan reports unchanged files, and source files remain untouched. **Note:** requires GUI environment; user should run the built executable and perform the smoke test.

---

## Self-Review

- Spec coverage: stable large-folder scanning, scan job state, cancellation, incremental scan, scan settings, and thumbnail status are each covered by a task.
- Placeholder scan: no task contains TBD or an unspecified implementation step.
- Type consistency: frontend ScanJob and Rust ScanJob use matching snake_case fields; command names match Tauri wrappers.
- Scope check: this plan does not add AI tagging, deletion, file move/rename, cloud sync, SVN, or full 3D/Spine runtime preview.
- Risk: Task 8 intentionally creates a thumbnail queue status foundation, while full concurrent background thumbnail workers can be implemented as the next plan if needed.

