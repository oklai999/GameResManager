# After Task 39 Next Stage Implementation Plan

**Progress Sync (2026-06-11):** Completed and verified. Selection UX and v0.3.1 Recent Activity are present in the current `0.7.0` codebase.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the immediate post-task39 UX correction, then implement v0.3.1 Recent Activity so users can return to recently opened, revealed, and copied assets.

**Architecture:** Keep all file-system actions behind Tauri commands. Recent activity is stored in SQLite as local app data and surfaced through typed React/Tauri wrappers; the original asset files are never deleted, moved, renamed, modified, or written to.

**Tech Stack:** Tauri 2, React 18, TypeScript, Vite, Rust, SQLite via `sqlx`, Vitest, cargo test.

---

## Source Context

- Work directory: `I:\GameResManger`
- Product spec: `G:\oklai999的策划仓库\游戏资源管理器\需求整理_v0.1.md`
- Roadmap plan: `G:\oklai999的策划仓库\游戏资源管理器\实施计划明细\2026-06-04-sharp-stock-reference-roadmap.md`
- Milestone tracker: `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`
- Current handoff assumption: Task 39 is complete. The next incomplete milestone is `v0.3.0 Follow-up Selection UX`, then `v0.3.1 Recent Activity`.

## Safety Rules

- Do not delete, move, rename, or modify original asset files.
- Do not write thumbnails, cache, database files, or metadata into source asset folders.
- Do not run batch deletion commands such as `del /s`, `rd /s`, `rmdir /s`, `Remove-Item -Recurse`, or `rm -rf`.
- If a task requires cleanup of many files, stop and ask the user for explicit instructions.

## File Structure

- `src/components/AssetGrid.tsx`: card selection behavior and explicit multi-select checkbox UI.
- `src/components/AssetGrid.test.tsx`: component tests for single-select and checkbox multi-select.
- `src/styles.css`: compact checkbox positioning and card selection styling.
- `src-tauri/migrations/0005_recent_asset_actions.sql`: SQLite table for local recent asset actions.
- `src-tauri/src/models.rs`: Rust model for recent action rows.
- `src/types/asset.ts`: TypeScript model for recent action rows.
- `src-tauri/src/db.rs`: repository functions to record and query recent actions.
- `src-tauri/src/commands.rs`: Tauri commands for recording and listing recent actions.
- `src-tauri/src/lib.rs`: command registration.
- `src/api/tauri.ts`: typed frontend wrappers.
- `src/App.tsx`: record successful safe actions and apply the recent filter.
- `src/components/LibrarySidebar.tsx`: add `最近使用` filter.
- `src/components/LibrarySidebar.test.tsx`: sidebar filter coverage.
- `tests/smoke/README.md`: manual smoke checks for selection and recent activity.

---

### Task 39A: Correct Selection UX

**Files:**
- Modify: `I:\GameResManger\src\components\AssetGrid.tsx`
- Modify: `I:\GameResManger\src\components\AssetGrid.test.tsx`
- Modify: `I:\GameResManger\src\styles.css`
- Modify: `I:\GameResManger\tests\smoke\README.md`
- Modify: `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`

- [x] **Step 1: Add failing selection tests**

Add these tests to `AssetGrid.test.tsx`:

```tsx
it("single-selects an asset when the card is clicked", async () => {
  const onSelectionChange = vi.fn();
  render(
    <AssetGrid
      assets={[
        asset,
        makeAsset({ id: 2, file_name: "tree.png", absolute_path: "C:/assets/tree.png" }),
      ]}
      selectedIds={[2]}
      onSelectionChange={onSelectionChange}
      onToggleFavorite={vi.fn()}
    />
  );

  await userEvent.click(screen.getByTitle("C:/assets/icon.png"));

  expect(onSelectionChange).toHaveBeenCalledWith([1]);
});

it("keeps the selected card selected when it is clicked again", async () => {
  const onSelectionChange = vi.fn();
  render(
    <AssetGrid
      assets={[asset]}
      selectedIds={[1]}
      onSelectionChange={onSelectionChange}
      onToggleFavorite={vi.fn()}
    />
  );

  await userEvent.click(screen.getByTitle("C:/assets/icon.png"));

  expect(onSelectionChange).toHaveBeenCalledWith([1]);
});

it("uses card checkboxes for multi-select", async () => {
  const onSelectionChange = vi.fn();
  render(
    <AssetGrid
      assets={[
        asset,
        makeAsset({ id: 2, file_name: "tree.png", absolute_path: "C:/assets/tree.png" }),
      ]}
      selectedIds={[1]}
      onSelectionChange={onSelectionChange}
      onToggleFavorite={vi.fn()}
    />
  );

  await userEvent.click(screen.getByRole("checkbox", { name: "选择 tree.png" }));

  expect(onSelectionChange).toHaveBeenCalledWith([1, 2]);
});

it("removes an asset from multi-select when its checkbox is unchecked", async () => {
  const onSelectionChange = vi.fn();
  render(
    <AssetGrid
      assets={[
        asset,
        makeAsset({ id: 2, file_name: "tree.png", absolute_path: "C:/assets/tree.png" }),
      ]}
      selectedIds={[1, 2]}
      onSelectionChange={onSelectionChange}
      onToggleFavorite={vi.fn()}
    />
  );

  await userEvent.click(screen.getByRole("checkbox", { name: "选择 tree.png" }));

  expect(onSelectionChange).toHaveBeenCalledWith([1]);
});
```

- [x] **Step 2: Verify tests fail for the current behavior**

Run:

```powershell
npm test -- src/components/AssetGrid.test.tsx
```

Expected: at least the single-select replacement test fails until the card click and checkbox selection paths are separated.

- [x] **Step 3: Separate single-select and multi-select handlers**

In `AssetGrid.tsx`, replace the existing `toggle` handler with:

```tsx
function selectOnly(assetId: number) {
  onSelectionChange([assetId]);
}

function toggleMulti(assetId: number, checked: boolean) {
  onSelectionChange(
    checked
      ? [...selectedIds.filter((id) => id !== assetId), assetId]
      : selectedIds.filter((id) => id !== assetId)
  );
}
```

Change the card click to:

```tsx
onClick={() => selectOnly(asset.id)}
```

- [x] **Step 4: Add explicit checkbox UI**

Add this input inside each `.asset-card`, before the thumbnail:

```tsx
<input
  type="checkbox"
  className="asset-select-checkbox"
  checked={selectedIds.includes(asset.id)}
  aria-label={`选择 ${asset.file_name}`}
  onClick={(event) => event.stopPropagation()}
  onChange={(event) => toggleMulti(asset.id, event.currentTarget.checked)}
/>
```

- [x] **Step 5: Style the checkbox**

Add compact CSS in `styles.css`:

```css
.asset-select-checkbox {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 3;
  width: 16px;
  height: 16px;
  margin: 0;
  accent-color: #5b8def;
}

.asset-card .thumb {
  position: relative;
}
```

If `.asset-card` is not already positioned, add:

```css
.asset-card {
  position: relative;
}
```

- [x] **Step 6: Add manual smoke checklist**

Append to `tests/smoke/README.md`:

```markdown
## Selection UX Smoke Test

- Click one asset card and confirm the right panel shows that single asset.
- Click another asset card and confirm selection changes to only the second asset.
- Click the selected asset card again and confirm it remains selected.
- Use checkboxes to select two assets and confirm the right panel switches to batch mode.
- Uncheck one selected asset and confirm it is removed from the batch selection.
- Confirm the favorite button does not change selection.
```

- [x] **Step 7: Run verification**

Run:

```powershell
npm test -- src/components/AssetGrid.test.tsx
npm run build
```

Expected: targeted tests pass and production build succeeds.

- [x] **Step 8: Update milestone tracker**

In the milestone tracker, set `v0.3.0 Follow-up Selection UX` to `Verified` and add completion evidence such as:

```markdown
| 2026-06-05 | v0.3.0 Follow-up Selection UX | Verified | `npm test -- src/components/AssetGrid.test.tsx`; `npm run build` | Card click is single-select; checkbox controls multi-select. |
```

- [x] **Step 9: Commit**

```powershell
git add src/components/AssetGrid.tsx src/components/AssetGrid.test.tsx src/styles.css tests/smoke/README.md
git commit -m "fix: separate single and multi asset selection"
```

If the planning repository at `G:\oklai999的策划仓库\游戏资源管理器` is a separate Git repo, commit its milestone tracker change there separately.

---

### Task 40: Add Recent Activity Schema and Models

**Files:**
- Create: `I:\GameResManger\src-tauri\migrations\0005_recent_asset_actions.sql`
- Modify: `I:\GameResManger\src-tauri\src\models.rs`
- Modify: `I:\GameResManger\src\types\asset.ts`

- [x] **Step 1: Create migration**

Create `src-tauri/migrations/0005_recent_asset_actions.sql`:

```sql
CREATE TABLE IF NOT EXISTS recent_asset_actions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  asset_id INTEGER NOT NULL,
  action_type TEXT NOT NULL CHECK (action_type IN ('open_file', 'reveal_folder', 'copy_path')),
  created_at TEXT NOT NULL,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recent_asset_actions_created_at
ON recent_asset_actions(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_recent_asset_actions_asset_id
ON recent_asset_actions(asset_id);
```

- [x] **Step 2: Add Rust model**

Add to `models.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecentAssetAction {
    pub id: i64,
    pub asset_id: i64,
    pub action_type: String,
    pub created_at: String,
}
```

- [x] **Step 3: Add TypeScript model**

Add to `asset.ts`:

```ts
export type RecentAssetAction = {
  id: number;
  asset_id: number;
  action_type: "open_file" | "reveal_folder" | "copy_path";
  created_at: string;
};
```

- [x] **Step 4: Run checks**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo check
cd I:\GameResManger
npm run build
```

Expected: Rust and TypeScript compile.

- [x] **Step 5: Commit**

```powershell
git add src-tauri/migrations/0005_recent_asset_actions.sql src-tauri/src/models.rs src/types/asset.ts
git commit -m "feat: add recent asset action model"
```

---

### Task 41: Record Safe File Actions

**Files:**
- Modify: `I:\GameResManger\src-tauri\src\db.rs`
- Modify: `I:\GameResManger\src-tauri\src\commands.rs`
- Modify: `I:\GameResManger\src-tauri\src\lib.rs`

- [x] **Step 1: Add repository tests**

Add a test in the existing `db.rs` tests module. Follow the local helper style already present in that file, but keep these assertions:

```rust
#[tokio::test]
async fn records_recent_asset_action() {
    let pool = test_pool().await;
    let folder = create_library_folder(&pool, "fixture", "C:/assets").await.unwrap();
    let asset = insert_test_asset(&pool, folder.id, "C:/assets/icon.png").await;

    record_recent_asset_action(&pool, asset.id, "copy_path").await.unwrap();
    let actions = list_recent_asset_actions(&pool, 10).await.unwrap();

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].asset_id, asset.id);
    assert_eq!(actions[0].action_type, "copy_path");
}
```

- [x] **Step 2: Implement repository functions**

Add to `db.rs`:

```rust
pub async fn record_recent_asset_action(
    pool: &SqlitePool,
    asset_id: i64,
    action_type: &str,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO recent_asset_actions (asset_id, action_type, created_at) VALUES (?1, ?2, ?3)",
    )
    .bind(asset_id)
    .bind(action_type)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_recent_asset_actions(
    pool: &SqlitePool,
    limit: i64,
) -> anyhow::Result<Vec<crate::models::RecentAssetAction>> {
    sqlx::query_as::<_, crate::models::RecentAssetAction>(
        "SELECT id, asset_id, action_type, created_at
         FROM recent_asset_actions
         ORDER BY created_at DESC
         LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}
```

- [x] **Step 3: Add commands**

Add to `commands.rs`:

```rust
#[tauri::command]
pub async fn record_recent_asset_action(
    db: State<'_, SqlitePool>,
    asset_id: i64,
    action_type: String,
) -> Result<(), CommandError> {
    match action_type.as_str() {
        "open_file" | "reveal_folder" | "copy_path" => {}
        _ => {
            return Err(CommandError {
                message: "unsupported recent action type".to_string(),
            })
        }
    }

    db::record_recent_asset_action(&*db, asset_id, &action_type)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn list_recent_asset_actions(
    db: State<'_, SqlitePool>,
    limit: i64,
) -> Result<Vec<crate::models::RecentAssetAction>, CommandError> {
    db::list_recent_asset_actions(&*db, limit)
        .await
        .map_err(Into::into)
}
```

Register both commands in `lib.rs`.

- [x] **Step 4: Run backend checks**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo test db::tests::records_recent_asset_action
cargo check
```

Expected: test and check pass.

- [x] **Step 5: Commit**

```powershell
git add src-tauri/src/db.rs src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: record recent asset actions"
```

---

### Task 42: Expose Recent Activity View

**Files:**
- Modify: `I:\GameResManger\src\api\tauri.ts`
- Modify: `I:\GameResManger\src\App.tsx`
- Modify: `I:\GameResManger\src\components\LibrarySidebar.tsx`
- Modify: `I:\GameResManger\src\components\LibrarySidebar.test.tsx`
- Modify: `I:\GameResManger\tests\smoke\README.md`
- Modify: `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`

- [x] **Step 1: Add API wrappers**

In `src/api/tauri.ts`, import `RecentAssetAction` and add:

```ts
export async function recordRecentAssetAction(
  assetId: number,
  actionType: RecentAssetAction["action_type"]
): Promise<void> {
  await invoke("record_recent_asset_action", { assetId, actionType });
}

export async function listRecentAssetActions(limit: number): Promise<RecentAssetAction[]> {
  return invoke<RecentAssetAction[]>("list_recent_asset_actions", { limit });
}
```

- [x] **Step 2: Add sidebar test**

Add to `LibrarySidebar.test.tsx`:

```tsx
it("shows recent activity filter", () => {
  render(<LibrarySidebar {...defaultProps} />);

  expect(screen.getByRole("button", { name: /最近使用/ })).toBeInTheDocument();
});
```

- [x] **Step 3: Add sidebar filter**

In `LibrarySidebar.tsx`, add a filter item near the top filters:

```tsx
{ id: "recent", label: "最近使用", icon: RotateCw },
```

- [x] **Step 4: Wire App state**

In `App.tsx`, add:

```ts
const [recentAssetIds, setRecentAssetIds] = useState<number[]>([]);
```

On load, fetch recent actions and derive unique IDs:

```ts
const actions = await listRecentAssetActions(100);
setRecentAssetIds([...new Set(actions.map((action) => action.asset_id))]);
```

After a safe action succeeds, record it:

```ts
await recordRecentAssetAction(asset.id, "copy_path");
setRecentAssetIds((prev) => [asset.id, ...prev.filter((id) => id !== asset.id)].slice(0, 100));
```

Use action types:

```ts
"open_file"
"reveal_folder"
"copy_path"
```

Apply the recent filter:

```ts
const displayAssets =
  activeFilter === "recent"
    ? gridAssets.filter((asset) => recentAssetIds.includes(asset.id))
    : gridAssets;
```

- [x] **Step 5: Keep failures non-blocking**

If recording recent activity fails after the file action succeeds, show a toast or console-safe error consistent with existing `App.tsx` error handling, but do not undo the file action.

- [x] **Step 6: Add manual smoke checklist**

Append to `tests/smoke/README.md`:

```markdown
## Recent Activity Smoke Test

- Open one asset from the details panel.
- Reveal another asset in its folder.
- Copy a third asset path.
- Click `最近使用` in the sidebar and confirm those assets appear.
- Restart the app and confirm recent activity is loaded from local app data.
- Confirm these actions do not delete, move, rename, or modify source files.
```

- [x] **Step 7: Run frontend checks**

Run:

```powershell
npm test -- src/components/LibrarySidebar.test.tsx
npm run build
```

Expected: test and build pass.

- [x] **Step 8: Run backend and full checks**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo test
cargo check
cd I:\GameResManger
npm test
npm run build
```

Expected: all automated checks pass. If `cargo` fails because of local MSVC or Windows SDK setup, record the exact environment failure and keep frontend checks as completed evidence.

- [x] **Step 9: Update milestone tracker**

Set `v0.3.1 Recent Activity` to `Verified` and add completion evidence:

```markdown
| 2026-06-05 | v0.3.1 Recent Activity | Verified | `cargo test`; `cargo check`; `npm test`; `npm run build` | Recent open/reveal/copy actions are recorded locally and exposed through `最近使用`. |
```

- [x] **Step 10: Commit**

```powershell
git add src/api/tauri.ts src/App.tsx src/components/LibrarySidebar.tsx src/components/LibrarySidebar.test.tsx tests/smoke/README.md
git commit -m "feat: add recent activity view"
```

If the planning repository at `G:\oklai999的策划仓库\游戏资源管理器` is a separate Git repo, commit its milestone tracker change there separately.

---

## Acceptance Criteria

- Normal asset card click selects exactly one asset and replaces previous selection.
- Clicking the selected card again keeps it selected.
- Multi-select is available through visible card checkboxes.
- Favorite clicks do not alter selection.
- `recent_asset_actions` records `open_file`, `reveal_folder`, and `copy_path` actions.
- `最近使用` appears in the sidebar and filters assets by recent action order.
- Recent activity persists in local SQLite app data.
- Original source files remain unchanged.

## Self-Review

- Spec coverage: This plan covers the immediate v0.3.0 selection UX follow-up and v0.3.1 Recent Activity milestone from the Sharp Stock roadmap.
- Placeholder scan: The plan contains concrete file paths, code snippets, commands, and expected results.
- Type consistency: Rust and TypeScript action types use the same string literals: `open_file`, `reveal_folder`, `copy_path`.
