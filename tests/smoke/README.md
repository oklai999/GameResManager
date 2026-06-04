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
