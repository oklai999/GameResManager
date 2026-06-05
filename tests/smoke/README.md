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
