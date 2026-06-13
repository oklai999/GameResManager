# Smoke QA

Use this checklist after implementing folder add and scan commands.

## Fixture

Create a local folder outside the repository with:

- `icon.png`
- `music.wav`
- `video.mp4`
- `font.ttf`
- `model.glb`
- `hero.skel`

## Checks

- Add the fixture folder as a resource folder.
- Run scan.
- Confirm resource cards appear in the grid.
- Confirm image files show thumbnails.
- Search by `icon` and confirm only `icon.png` remains visible.
- Filter by image and confirm image assets remain visible.
- Select one asset and confirm the details panel shows path, type, size, and action buttons.
- Select two assets and confirm the right panel switches to batch mode.
- Apply tag `待整理` to selected assets.
- Copy path and confirm clipboard contains the absolute file path.
- Open file and open containing folder from the details panel.

## Safety

The app must not delete, move, rename, or modify fixture files during this smoke test.

---

## Folder Management + Sticky Details Panel Smoke Test

Use a fixture folder with known assets.

- [ ] Add a fixture folder and run scan.
- [ ] Confirm the folder row shows resource count, missing count (if any), and last scanned time.
- [ ] Confirm the folder row shows an "打开" button.
- [ ] Click "打开" and confirm the source folder opens in File Explorer.
- [ ] Click "扫描" and confirm scanning starts.
- [ ] Confirm "取消" button appears during scanning.
- [ ] Click "取消" and confirm scan stops.
- [ ] Scroll the center asset grid and confirm the left sidebar and right details panel remain fixed (do not scroll with the grid).
- [ ] Select an asset with long metadata and confirm the right details panel scrolls internally.
- [ ] Click "从资源库移除索引" (✕) and confirm a confirmation dialog appears with text stating only the index will be removed, not the source files.
- [ ] Cancel the removal and confirm the folder and its assets remain visible.
- [ ] Confirm removal and confirm the folder disappears from the sidebar, but the source folder still exists on disk.
- [ ] Confirm already-deleted folder's assets no longer appear in searches or filters.

## Folder Manager Backup Cleanup Smoke Test

Use this before backing up the project directory when you want the app index to forget test resource folders.

- [ ] Add at least two fixture folders and scan one of them.
- [ ] Click `管理` next to `素材文件夹`.
- [ ] Confirm the `资源库文件夹管理` panel lists folder name, absolute path, resource count, missing count, accessibility, and last scan time.
- [ ] Click `打开文件夹` and confirm the source folder opens in File Explorer.
- [ ] Click `从资源库移除` for one folder and confirm the warning says only app indexes and related organization data are removed.
- [ ] Confirm removal and verify the folder disappears from the sidebar and search results.
- [ ] Confirm the source folder and its files still exist on disk.
- [ ] Start scanning another folder, open `管理`, and confirm its remove action is disabled until the scan is cancelled.
- [ ] Confirm no source file is deleted, moved, renamed, modified, or written to during this smoke test.

---

## Thumbnail Display Smoke Test

Use a fixture folder containing both real images and macOS metadata artifacts.

- [ ] Add a folder that contains:
  - `real_icon.png` (valid PNG image)
  - `corrupt.png` (invalid/corrupted image file)
  - `._fake.png` (macOS AppleDouble metadata file)
  - `.DS_Store` (macOS folder metadata)
  - `__MACOSX/._real_icon.png` (macOS resource fork)
- [ ] Run scan.
- [ ] Confirm `real_icon.png` shows a thumbnail in the grid (not "无预览").
- [ ] Confirm `corrupt.png` appears in the grid with status "缩略图失败".
- [ ] Confirm `._fake.png`, `.DS_Store`, and `__MACOSX/._real_icon.png` do **not** appear in the grid.
- [ ] Confirm `scan_jobs.skipped_count` accounts for the ignored metadata files.
- [ ] Select `real_icon.png` and confirm the details panel shows the image preview.
- [ ] Delete the generated `.webp` thumbnail file from the cache directory while keeping the DB record as `ready`.
- [ ] Restart the app and confirm the same asset now shows "预览加载失败" (onError fallback).
- [ ] Run a second scan on the same folder without modifying `real_icon.png`.
- [ ] Confirm the thumbnail for `real_icon.png` remains visible (unchanged file preserves thumbnail).

## Large Folder Release Scan

Use `G:\资源\2D游戏资源_淘宝\` or equivalent large local folder (about 62,000+ supported files).

- [ ] Add folder.
- [ ] Click Scan.
- [ ] Confirm Cancel button appears within one second.
- [ ] Confirm `scan_jobs` table has a `running` row.
- [ ] Confirm asset count in grid increases during scan (not stuck at 0).
- [ ] Click Cancel and confirm scan stops and job status becomes `cancelled`.
- [ ] Confirm already-inserted assets remain visible after cancellation.
- [ ] Run second scan on same folder and confirm `unchanged_count` is reported.
- [ ] Confirm ignored directories (`node_modules`, `.git`, `.godot`, `target`, `dist`, `build`, `.codex_spreadsheet_tinyswords`) are skipped.
- [ ] Confirm PSD files are indexed but do not generate thumbnails by default.
- [ ] Confirm source asset files are untouched (no delete, move, rename, or modification).

---

## Type Coverage + Placeholder Smoke Test

Use a fixture folder with the newly covered formats.

- [ ] Create or choose a local fixture folder outside the repository with:
  - `concept.psd`
  - `hero.spine`
  - `music.aac`
  - `loop.m4a`
  - `preview.mkv`
  - `mesh.usdz`
- [ ] Add the folder as a resource library.
- [ ] Run scan and wait for completion.
- [ ] Confirm every file appears in the grid with the correct type badge.
- [ ] Confirm PSD cards show placeholder text **PSD** (not "无预览").
- [ ] Confirm Spine cards show placeholder text **Spine** and badge label **Spine**.
- [ ] Confirm 3D/USDZ cards show placeholder text **3D**.
- [ ] Confirm AAC and M4A files are classified as **audio**.
- [ ] Confirm MKV files are classified as **video**.
- [ ] Confirm plain image assets without thumbnails show placeholder text **图片** (not "无预览").
- [ ] Confirm search, filter, and details panel do not fall back to vague "无预览" for any of the above types.
- [ ] Select each file and confirm the details panel still shows path, size, modified time, open file, open folder, and copy path.
- [ ] Confirm no source file is deleted, moved, renamed, or modified.

## Selection UX Smoke Test

- Click one asset card and confirm the right panel shows that single asset.
- Click another asset card and confirm selection changes to only the second asset.
- Click the selected asset card again and confirm it remains selected.
- Use checkboxes to select two assets and confirm the right panel switches to batch mode.
- Uncheck one selected asset and confirm it is removed from the batch selection.
- Confirm the favorite button does not change selection.

## Recent Activity Smoke Test

- Open one asset from the details panel.
- Reveal another asset in its folder.
- Copy a third asset path.
- Click `最近使用` in the sidebar and confirm those assets appear.
- Restart the app and confirm recent activity is loaded from local app data.
- Confirm these actions do not delete, move, rename, or modify source files.

## Tag Efficiency Smoke Test

- Select one asset and add tag `地形`.
- Select another asset and confirm `地形` appears as a suggested tag.
- Type `地` in the tag input and confirm suggestions narrow to matching tags.
- Click suggested tag `地形` and confirm it is applied.
- Select two assets and confirm the batch panel shows only common tags as chips.
- Add tag `批量整理` to the batch selection and confirm both selected assets receive it after refresh.
- Confirm already-applied tags are not suggested again for the current selection.
- Restart the app and confirm recently used tags still appear as suggestions.
- Confirm no source file is deleted, moved, renamed, or modified.

---

## Path Friendly Display Smoke Test

- Select one asset and confirm the absolute path remains visible in the details panel.
- Click `复制路径` and confirm the clipboard contains the absolute path.
- Click `复制正斜杠路径` and confirm backslashes are converted to `/`.
- Click `复制文件夹路径` and confirm the clipboard contains the parent folder path.
- Click `复制文件名` and confirm the clipboard contains only the file name.
- Set an explicit project root that contains the selected asset.
- Confirm the details panel shows a valid `res://...` path.
- Copy the `res://...` path and confirm the clipboard value.
- Set or choose a project root that does not contain the selected asset.
- Confirm the app does not generate an invalid `res://...` path.
- Confirm `打开文件` and `打开所在目录` still work.
- Confirm no source file is deleted, moved, renamed, or modified.

---

## Advanced Capability Gate Smoke Review

- Confirm the app does not show AI auto-tagging actions in the main workbench.
- Confirm the app does not show batch delete, batch move, or batch rename actions.
- Confirm the app does not show texture atlas export actions.
- Confirm the app does not require network access for search, preview, tags, favorites, collections, notes, recent activity, or path copying.
- Confirm non-previewable 3D, Spine, PSD, audio, and video files still show safe actions: open file, open containing folder, and copy path.
- Confirm no source file is deleted, moved, renamed, modified, or written to during this review.

---

## Advanced Filters And Sorting Smoke Test

- Scan or choose a library containing at least three assets with different file sizes.
- Set a minimum file size and confirm smaller assets disappear from the grid.
- Set a maximum file size and confirm larger assets disappear from the grid.
- Use width or height filters on image assets and confirm assets without matching dimensions are hidden.
- Set a modified-date range and confirm only assets inside the range remain visible.
- Sort by name ascending and confirm card order follows file name.
- Sort by size descending and confirm larger files appear before smaller files.
- Sort by modified time descending and confirm recently modified files appear first.
- Clear advanced filters and confirm the full filtered result set returns.
- Confirm no source file is deleted, moved, renamed, modified, or written to during this smoke test.

---

## Large Library Pagination Smoke Test

- Scan or choose a library with more than 250 indexed assets.
- Confirm the first page appears without waiting for all matching assets to render.
- Confirm the footer shows `已显示 200 / 总数` or the current page size equivalent.
- Click `加载更多` and confirm additional cards append without clearing selection unexpectedly.
- Change a search query, filter, or sort option and confirm results reset to the first page.
- Confirm favorite, tag, collection, details, open file, open folder, and copy path still work on loaded pages.
- Confirm no source file is deleted, moved, renamed, modified, or written to during this smoke test.

---

## v0.6.0 Virtualized Grid And FTS Search Smoke Test

- Scan or choose a library with at least 2,000 indexed assets.
- Confirm the grid scrolls smoothly and does not render thousands of card nodes at once.
- Confirm selection, checkbox multi-select, favorite toggle, details panel, tags, collections, open file, reveal folder, and copy path still work on cards after scrolling deep into the list.
- Search by file name and confirm matching assets appear.
- Add or edit a note, search by that note text, and confirm the asset appears.
- Add a tag, search by that tag text, and confirm the asset appears.
- For a Chinese note such as `主角待机`, search `主角` and confirm the asset appears (prefix match). Search `待机` and confirm it does **not** appear (arbitrary CJK substring search is not supported in v0.6.0).
- Change type, size, dimension, modified-time, favorite, missing, folder, collection, and sort filters and confirm pagination count remains coherent.
- Confirm no source file is deleted, moved, renamed, modified, or written to during this smoke test.

---

## v0.7.0 CJK Substring Search Smoke Test

- Use a library containing an asset whose note is `主角待机动画`.
- Search `待机` with only "备注" enabled and confirm the asset appears.
- Search `背景树` for a file named `森林背景树.png` with only "文件名" enabled and confirm it appears.
- Search `角色` with only "路径" enabled and confirm file-name/note/tag-only matches do not appear.
- Search `主角 动画` and confirm every result matches both terms across enabled scopes.
- Search literal `%`, `_`, `"`, `AND`, `OR`, and `NOT` text and confirm the app does not error.
- On a library with at least 20,000 indexed assets, compare a three-character query and a two-character query. Confirm the UI stays responsive and record approximate result time in the test notes.
- Edit a note and add a tag, then immediately search for a middle substring from each value.
- Rescan the folder and confirm the same substring searches still work.
- Remove a library folder from the app and confirm its former assets no longer appear in search; confirm source files remain on disk.

---

## v0.8.0 Collection Management Smoke Test

Status: Passed manually on 2026-06-11.

---

## v0.9.0 Tag Management Smoke Test

- [ ] Create or apply tag `角色` to two assets.
- [ ] Open tag management and confirm `角色` shows an asset count of `2`.
- [ ] Rename `角色` to `主角`, change its color to `#22AA88`, and confirm both values persist after reopening the manager.
- [ ] Search for `主角` in tag scope and confirm the two assets are returned.
- [ ] Create tag `角色动画`, apply it to one of the same assets, then rename `主角` to `角色动画`.
- [ ] Confirm the two tags merge into one, the target color is preserved, the asset count has no duplicates, and both assets remain searchable.
- [ ] Select one asset and remove `角色动画`; confirm only that resource loses the tag and the tag itself remains.
- [ ] Apply the tag to two assets, select both, and remove the common tag; confirm both relationships are removed.
- [ ] Delete the tag through the confirmation flow and confirm the tag disappears.
- [ ] Confirm all source assets still exist at the same paths with unchanged contents and modified times.
- [ ] Confirm tag deletion, merge, and unlink never delete, move, rename, or modify source files.

- Create a collection named `角色` and confirm it appears with `0` resources.
- Select two assets and add them to `角色`; confirm the collection count becomes `2`.
- Select the `角色` collection and confirm only its members appear.
- Open collection management, rename it to `主角`, add description `常用角色素材`, close and reopen the manager, and confirm both values persist.
- While filtering by `主角`, select one resource and use `从当前集合移出`; confirm the resource disappears and the count decreases.
- Select multiple remaining resources and use the batch remove action; confirm all selected links are removed.
- Add the same asset to the same collection twice and confirm no duplicate card or count is created.
- Delete the collection and confirm the collection filter clears.
- Confirm every source asset still exists at the same absolute path and its contents and modified time are unchanged.
- Confirm no source file is deleted, moved, renamed, modified, or written to during this smoke test.

---

## v0.9.1 Focus Workbench UI Smoke Test

- [ ] Confirm the left navigation rail shows 资源库、类型、标签、集合、最近、设置.
- [ ] Click each rail item and confirm the contextual sidebar shows only the matching content.
- [ ] Click the active rail item again and confirm the contextual sidebar collapses without hiding the grid or inspector.
- [ ] Search by file name and confirm results update as before.
- [ ] Open 筛选 and confirm search scopes, size, dimensions, date range and reset controls remain functional.
- [ ] Apply at least three advanced filters and confirm readable filter chips appear below the toolbar.
- [ ] Remove one chip and confirm only that filter is cleared.
- [ ] Change sorting and confirm result order changes.
- [ ] Switch between 紧凑网格 and 舒适网格 and confirm selection, favorites and virtual scrolling remain functional.
- [ ] Select one asset and confirm preview, properties, path variants, note, tags and collections remain available.
- [ ] Scroll a long inspector and confirm 打开文件、所在目录、复制路径 remain fixed at the bottom.
- [ ] Select multiple assets and confirm batch tags, collections and current-collection removal remain available.
- [ ] Run a scan and confirm scan status remains readable without covering search or results.
- [ ] Resize the window down to 1100px width and confirm all four regions remain usable without overlapping.
- [ ] Confirm keyboard focus is visible on rail, toolbar, cards, inspector controls and dialogs.
- [ ] Confirm no source file is deleted, moved, renamed, modified or written to.

### v0.9.1 Release Verification Record

Verification run on 2026-06-13:

- [x] Development desktop application starts successfully through `npm run tauri dev`.
- [x] Vite reports ready, Rust finishes the dev build, and `game-resource-manager.exe` remains responsive.
- [x] Startup logs contain no blank-screen, panic, or runtime command error.
- [x] Frontend, Rust, production build, Tauri bundle, and release packaging checks pass.
- [ ] Repeat the interactive checks above on the final package. Automated Windows desktop control was unavailable because the installed `@oai/sky` package does not export its Computer Use client module.

The unchecked interaction items remain the final manual acceptance checklist. Startup success is not treated as proof that every visual state and interaction passed.
