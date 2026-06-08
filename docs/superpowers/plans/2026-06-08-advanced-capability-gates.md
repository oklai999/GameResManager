# Advanced Capability Gates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete v0.4 planning by documenting decision gates for high-risk Sharp Stock-style capabilities before any implementation begins.

**Architecture:** This stage is documentation and product-risk work only. It creates explicit entry criteria for AI auto-tagging, texture atlas tools, rich 3D/Spine/audio previews, and cloud/team features while preserving the local-first, source-file-safe architecture already used by the Tauri + React + SQLite app.

**Tech Stack:** Markdown planning docs, existing milestone tracker, project safety rules, optional repository status checks with Git.

---

## Source Context

- Work directory: `I:\GameResManger`
- Product spec: `G:\oklai999的策划仓库\游戏资源管理器\需求整理_v0.1.md`
- Roadmap plan: `G:\oklai999的策划仓库\游戏资源管理器\实施计划明细\2026-06-04-sharp-stock-reference-roadmap.md`
- Milestone tracker: `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`
- Current milestone state checked on 2026-06-08:
  - `v0.3.0 Type Coverage + Manageable Placeholders`: `Verified`
  - `v0.3.0 Follow-up Selection UX`: `Verified`
  - `v0.3.1 Recent Activity`: `Verified`
  - `v0.3.2 Tag Efficiency`: `Verified`
  - `v0.3.3 Path Friendly Display`: `Verified`
  - `v0.4 Advanced Capability Gates`: `Planned`

## Safety Rules

- Do not delete, move, rename, or modify original asset files.
- Do not write thumbnails, caches, databases, or metadata into source asset folders.
- Do not add implementation for AI auto-tagging, atlas tools, full runtime previews, cloud sync, or team backend in this stage.
- Do not run batch deletion commands such as `del /s`, `rd /s`, `rmdir /s`, `Remove-Item -Recurse`, or `rm -rf`.
- If cleanup appears necessary, stop and ask the user for explicit instructions.

## File Structure

- Create: `G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md`
  - Owns v0.4 decision gates and explicit non-goals for high-risk features.
- Modify: `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`
  - Marks `v0.4 Advanced Capability Gates` as `Documented` after the decision record exists.
- Modify: `I:\GameResManger\README.md`
  - Adds a short roadmap note only if the README already has a roadmap or release-notes section where this naturally belongs.
- Modify: `I:\GameResManger\tests\smoke\README.md`
  - Adds a manual safety review checklist for confirming high-risk abilities remain gated and absent from the app UI.

---

### Task 50: Create Advanced Capability Gate Document

**Files:**
- Create: `G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md`

- [ ] **Step 1: Ensure the research directory exists**

Run:

```powershell
New-Item -ItemType Directory -Path 'G:\oklai999的策划仓库\游戏资源管理器\研究记录' -Force
```

Expected: the directory exists. This command must not delete or modify source asset files.

- [ ] **Step 2: Create the decision record**

Create `G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md` with this exact content:

```markdown
# Advanced Capability Gates

**Date:** 2026-06-08
**Product:** 游戏资源管理器 / Game Resource Manager
**Status:** v0.4 decision record

## Purpose

This document keeps high-risk Sharp Stock-style capabilities outside active implementation until they satisfy local-first, source-file-safe, and user-consent requirements.

## Global Rules

- The app must not delete, move, rename, or modify original source assets.
- The app must not write cache, database, metadata, or generated files into source asset folders.
- Any capability that sends asset content outside the local machine requires explicit user approval before implementation.
- Any generated output must be written only to a user-chosen output path or app data path.
- Indexing, search, open file, reveal folder, copy path, tags, favorites, collections, notes, and recent activity must continue working if an advanced preview or automation feature fails.

## AI Auto Tagging

**Current status:** Excluded from active implementation.

**Allowed research questions:**

- Can tags be generated fully locally with acceptable speed on a normal Windows desktop?
- If a remote model is used, what asset data would leave the machine?
- Can every generated tag be shown as a suggestion that requires user approval before saving?

**Entry criteria before implementation:**

- The user can disable the feature completely.
- The app never uploads source assets without explicit opt-in consent.
- The UI clearly distinguishes suggested tags from user-applied tags.
- Generated tags are stored only after user approval.
- Failed tagging never blocks scanning or normal asset management.

**Minimum prototype shape after approval:**

- A local-only experiment command that reads app-indexed metadata first.
- A review queue where the user accepts or rejects suggested tags.
- No automatic mutation of existing tags.

## Texture Atlas Tools

**Current status:** Separate tool candidate, not part of v0.3 or v0.4 implementation.

**Allowed research questions:**

- Which atlas formats matter first for the user's Godot or other engine workflow?
- Should atlas generation be a separate export tool rather than part of the asset manager core?
- What preview and dry-run output is needed before writing generated files?

**Entry criteria before implementation:**

- Source image files are never overwritten.
- The output directory is explicitly chosen by the user.
- The app shows a dry-run summary before writing output.
- Generated atlas files have collision-safe names.
- The operation can be cancelled before writing begins.

**Minimum prototype shape after approval:**

- Select assets from the index.
- Preview atlas layout and output file names.
- Export generated files to a user-selected folder.

## Full 3D, Spine, And Audio Workstation Preview

**Current status:** Research candidate after v0.3.

**Allowed research questions:**

- Which preview libraries are small and stable enough for a Tauri desktop app?
- Can heavy preview runtimes be loaded lazily after asset selection?
- What fallback UI is needed when a preview format fails?

**Entry criteria before implementation:**

- Preview dependencies are isolated from scanning and search.
- A preview failure never prevents an asset from being indexed.
- Non-previewable files still show type, size, path, open file, reveal folder, and copy path actions.
- Large preview files cannot freeze the main UI thread.
- The app has tests or smoke checks proving normal grid browsing remains fast.

**Minimum prototype shape after approval:**

- A right-panel preview experiment for one narrow file type.
- Lazy loading only after user selection.
- Explicit fallback state on preview failure.

## Cloud Sync Or Team Backend

**Current status:** Out of scope for this local-first product line.

**Allowed research questions:**

- Is there a separate product need for team metadata sync that does not touch source assets?
- Can an export/import workflow satisfy collaboration needs without a backend?

**Entry criteria before implementation:**

- A separate privacy model exists.
- Local search and preview do not depend on network availability.
- Users can keep the application fully offline.
- The app never silently uploads paths, metadata, thumbnails, or source assets.

**Minimum prototype shape after approval:**

- None for the current local-first roadmap.

## v0.4 Decision

No advanced capability should enter implementation until the user explicitly selects it as a new phase and confirms the safety boundary for that phase.
```

- [ ] **Step 3: Review the document for prohibited scope**

Run:

```powershell
Select-String -LiteralPath 'G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md' -Pattern 'Current status|Entry criteria|No advanced capability'
```

Expected: the output shows decision-gate sections and does not imply implementation has started.

- [ ] **Step 4: Commit or record planning-doc status**

Run:

```powershell
git -C 'G:\oklai999的策划仓库\游戏资源管理器' status --short
```

Expected: if that planning directory is a Git repository, the new research document appears as an untracked or modified file. If it is not a Git repository, record that the document exists and continue.

---

### Task 51: Update Milestone Tracker

**Files:**
- Modify: `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`

- [ ] **Step 1: Change the v0.4 board status**

In the milestone board row for `v0.4 Advanced Capability Gates`, change:

```markdown
| v0.4 Advanced Capability Gates | Planned | Keep high-risk reference abilities outside implementation until they satisfy local-first and safety criteria. | Decision record for AI auto-tagging, texture atlas tools, full 3D/Spine/audio preview, cloud/team backend. | Review `G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-04-advanced-capability-gates.md`. | No implementation work planned. |
```

to:

```markdown
| v0.4 Advanced Capability Gates | Documented | Keep high-risk reference abilities outside implementation until they satisfy local-first and safety criteria. | Decision record for AI auto-tagging, texture atlas tools, full 3D/Spine/audio preview, cloud/team backend. | Review `G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md`. | Decision gates documented; no advanced capability implementation started. |
```

- [ ] **Step 2: Add completion log entry**

Append this row to the completion log table:

```markdown
| 2026-06-08 | v0.4 Advanced Capability Gates | Documented | Review `G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md` | AI auto-tagging, atlas tools, rich previews, and cloud/team backend remain gated behind explicit safety criteria. |
```

- [ ] **Step 3: Verify the milestone points to the new record**

Run:

```powershell
Select-String -LiteralPath 'G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md' -Pattern '2026-06-08-advanced-capability-gates|Documented'
```

Expected: the milestone row and completion log both reference the 2026-06-08 decision record.

---

### Task 52: Add Safety Smoke Review

**Files:**
- Modify: `I:\GameResManger\tests\smoke\README.md`

- [ ] **Step 1: Append high-risk capability gate checks**

Append this section to `I:\GameResManger\tests\smoke\README.md`:

```markdown
## Advanced Capability Gate Smoke Review

- Confirm the app does not show AI auto-tagging actions in the main workbench.
- Confirm the app does not show batch delete, batch move, or batch rename actions.
- Confirm the app does not show texture atlas export actions.
- Confirm the app does not require network access for search, preview, tags, favorites, collections, notes, recent activity, or path copying.
- Confirm non-previewable 3D, Spine, PSD, audio, and video files still show safe actions: open file, open containing folder, and copy path.
- Confirm no source file is deleted, moved, renamed, modified, or written to during this review.
```

- [ ] **Step 2: Run documentation diff**

Run:

```powershell
git diff -- tests/smoke/README.md
```

Expected: the diff only adds the `Advanced Capability Gate Smoke Review` section.

- [ ] **Step 3: Commit smoke documentation**

Run:

```powershell
git add tests/smoke/README.md
git commit -m "docs: add advanced capability gate smoke review"
```

Expected: a commit is created for the smoke checklist update. Do not stage unrelated release artifacts or unrelated deleted files.

---

### Task 53: Final v0.4 Verification

**Files:**
- No implementation files.

- [ ] **Step 1: Run frontend checks**

Run:

```powershell
npm test
npm run build
```

Expected: all frontend tests pass and production build succeeds. This confirms the documentation-only stage did not break the local app.

- [ ] **Step 2: Run backend checks**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo test
cargo check
```

Expected: all Rust tests pass and backend compiles. If local MSVC or Windows SDK configuration fails, record the exact environment error and do not change business code to work around it.

- [ ] **Step 3: Review working tree scope**

Run:

```powershell
git status --short
```

Expected: only v0.4 documentation changes are present in `I:\GameResManger`. Existing unrelated deleted release files or unrelated untracked files must not be staged or reverted.

- [ ] **Step 4: Record planning repository status**

Run:

```powershell
git -C 'G:\oklai999的策划仓库\游戏资源管理器' status --short
```

Expected: if the planning directory is a Git repository, it shows the research and milestone document changes. If it is not a Git repository, the files still exist in the requested planning directory.

---

## Acceptance Criteria

- A v0.4 decision record exists at `G:\oklai999的策划仓库\游戏资源管理器\研究记录\2026-06-08-advanced-capability-gates.md`.
- The milestone tracker marks `v0.4 Advanced Capability Gates` as `Documented`.
- AI auto-tagging remains excluded from implementation until explicit user approval and safety criteria exist.
- Texture atlas tools remain excluded from implementation until explicit user approval and output-safety criteria exist.
- Full 3D, Spine, and audio workstation previews remain excluded from implementation until lazy-loading and failure-isolation criteria exist.
- Cloud sync and team backend remain out of scope for the local-first product line.
- The smoke checklist includes a manual review proving high-risk abilities are still gated.
- No source asset file is deleted, moved, renamed, modified, or written to.

## Self-Review

- Spec coverage: Covers the remaining `v0.4 Advanced Capability Gates` row from the Sharp Stock roadmap and the excluded capabilities listed in the product spec section 14.3.
- Placeholder scan: The plan contains exact file paths, document content, commands, expected results, and acceptance criteria.
- Type consistency: This stage adds no code types or command names, so there are no Rust/TypeScript contracts to synchronize.
