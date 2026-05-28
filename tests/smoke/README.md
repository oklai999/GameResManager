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
