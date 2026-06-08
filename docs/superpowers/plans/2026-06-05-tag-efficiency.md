# Tag Efficiency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve tag workflow speed by exposing recent tags, searchable suggestions, and batch-friendly tag application.

**Architecture:** Keep tag persistence in Rust/SQLite and expose it through typed Tauri commands. React components remain UI-focused: `App.tsx` loads recent tag data, `DetailsPanel` passes it into `TagEditor`, and `TagEditor` handles local filtering and suggestion clicks.

**Tech Stack:** Tauri 2, React 18, TypeScript, Vite, Rust, SQLite via `sqlx`, Vitest, cargo check/test.

---

## Source Context

- Work directory: `I:\GameResManger`
- Product spec: `G:\oklai999的策划仓库\游戏资源管理器\需求整理_v0.1.md`
- Roadmap: `G:\oklai999的策划仓库\游戏资源管理器\实施计划明细\2026-06-04-sharp-stock-reference-roadmap.md`
- Milestone tracker: `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`
- Current state: v0.3.1 Recent Activity is verified. Next milestone is `v0.3.2 Tag Efficiency`.

## Safety Rules

- Do not delete, move, rename, or modify original asset files.
- Do not write caches, metadata, thumbnails, or database files into source asset folders.
- Do not add delete, batch move, or batch rename features.
- Do not run batch deletion commands such as `del /s`, `rd /s`, `rmdir /s`, `Remove-Item -Recurse`, or `rm -rf`.

## File Structure

- `src-tauri/src/db.rs`: add `list_recent_tags` repository query and focused tests.
- `src-tauri/src/commands.rs`: add `list_recent_tags` Tauri command.
- `src-tauri/src/lib.rs`: register the new command.
- `src/api/tauri.ts`: add typed `listRecentTags` wrapper.
- `src/components/TagEditor.tsx`: accept `recentTags`, merge recent/all tags, filter suggestions, and apply suggestion clicks.
- `src/components/TagEditor.test.tsx`: cover filtering, recent-first ordering, exclusion of existing tags, and suggestion click behavior.
- `src/components/DetailsPanel.tsx`: accept `recentTags` and pass them into both single and batch `TagEditor` instances.
- `src/components/DetailsPanel.test.tsx`: keep existing tests compiling with the new optional prop.
- `src/App.tsx`: load recent tags, refresh them after applying a tag, and pass them into `DetailsPanel`.
- `tests/smoke/README.md`: add manual tag efficiency smoke checks.
- Milestone tracker: mark v0.3.2 verified after implementation and validation.

---

### Task 43: Expose Recent Tags

**Files:**
- Modify: `I:\GameResManger\src-tauri\src\db.rs`
- Modify: `I:\GameResManger\src-tauri\src\commands.rs`
- Modify: `I:\GameResManger\src-tauri\src\lib.rs`
- Modify: `I:\GameResManger\src\api\tauri.ts`

- [ ] **Step 1: Add repository tests**

Add this test module near existing `db.rs` tag or collection tests:

```rust
#[cfg(test)]
mod recent_tag_tests {
    use super::*;

    async fn setup_db() -> Db {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#5B8DEF',
                created_at TEXT NOT NULL,
                last_used_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn lists_recent_tags_by_last_used_then_name() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO tags (name, color, created_at, last_used_at) VALUES ('角色', '#5B8DEF', '2026-01-01T00:00:00Z', '2026-01-03T00:00:00Z')")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO tags (name, color, created_at, last_used_at) VALUES ('地形', '#5B8DEF', '2026-01-01T00:00:00Z', '2026-01-04T00:00:00Z')")
            .execute(&db).await.unwrap();
        sqlx::query("INSERT INTO tags (name, color, created_at, last_used_at) VALUES ('特效', '#5B8DEF', '2026-01-01T00:00:00Z', '2026-01-03T00:00:00Z')")
            .execute(&db).await.unwrap();

        let tags = list_recent_tags(&db, 3).await.unwrap();

        let names: Vec<String> = tags.into_iter().map(|tag| tag.name).collect();
        assert_eq!(names, vec!["地形", "特效", "角色"]);
    }

    #[tokio::test]
    async fn clamps_recent_tag_limit() {
        let db = setup_db().await;
        sqlx::query("INSERT INTO tags (name, color, created_at, last_used_at) VALUES ('地形', '#5B8DEF', '2026-01-01T00:00:00Z', '2026-01-04T00:00:00Z')")
            .execute(&db).await.unwrap();

        let tags = list_recent_tags(&db, -5).await.unwrap();

        assert!(tags.is_empty());
    }
}
```

- [ ] **Step 2: Run the failing tests**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo test recent_tag_tests
```

Expected: fail because `list_recent_tags` does not exist yet.

- [ ] **Step 3: Implement repository function**

Add to `db.rs`:

```rust
pub async fn list_recent_tags(db: &Db, limit: i64) -> anyhow::Result<Vec<crate::models::Tag>> {
    let limit = limit.clamp(0, 50);
    sqlx::query_as::<_, crate::models::Tag>(
        "SELECT id, name, color
         FROM tags
         ORDER BY last_used_at DESC, name ASC
         LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}
```

- [ ] **Step 4: Add Tauri command**

Add to `commands.rs`:

```rust
#[tauri::command]
pub async fn list_recent_tags(
    db: State<'_, SqlitePool>,
    limit: i64,
) -> Result<Vec<crate::models::Tag>, CommandError> {
    db::list_recent_tags(&*db, limit).await.map_err(Into::into)
}
```

Register `commands::list_recent_tags` in `src-tauri/src/lib.rs`.

- [ ] **Step 5: Add frontend API wrapper**

Add to `src/api/tauri.ts`:

```ts
export async function listRecentTags(limit: number): Promise<Tag[]> {
  return invoke<Tag[]>("list_recent_tags", { limit });
}
```

- [ ] **Step 6: Run verification**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo test recent_tag_tests
cargo check
cd I:\GameResManger
npm run build
```

Expected: tests pass, backend compiles cleanly, frontend build succeeds.

- [ ] **Step 7: Commit**

```powershell
git add src-tauri/src/db.rs src-tauri/src/commands.rs src-tauri/src/lib.rs src/api/tauri.ts
git commit -m "feat: expose recent tags"
```

---

### Task 44: Add Tag Suggestions To TagEditor

**Files:**
- Modify: `I:\GameResManger\src\components\TagEditor.tsx`
- Modify: `I:\GameResManger\src\components\TagEditor.test.tsx`

- [ ] **Step 1: Update the API mock**

In `TagEditor.test.tsx`, change the mock so each test can control tags:

```tsx
import { listTags } from "../api/tauri";

vi.mock("../api/tauri", () => ({
  listTags: vi.fn(),
}));

const mockedListTags = vi.mocked(listTags);
```

Before each test:

```tsx
beforeEach(() => {
  mockedListTags.mockResolvedValue([]);
});
```

- [ ] **Step 2: Add failing behavior tests**

Add these tests:

```tsx
it("shows recent tags before older tag suggestions", async () => {
  mockedListTags.mockResolvedValue([
    { id: 1, name: "角色", color: "#5B8DEF" },
    { id: 2, name: "地形", color: "#5B8DEF" },
  ]);

  render(<TagEditor existingTags={[]} recentTags={["特效", "角色"]} onApply={vi.fn()} />);

  await screen.findByRole("button", { name: "特效" });
  const suggestions = screen.getAllByRole("button").map((button) => button.textContent);

  expect(suggestions).toContain("特效");
  expect(suggestions.indexOf("特效")).toBeLessThan(suggestions.indexOf("角色"));
});

it("filters tag suggestions by typed text", async () => {
  mockedListTags.mockResolvedValue([
    { id: 1, name: "地形", color: "#5B8DEF" },
    { id: 2, name: "角色", color: "#5B8DEF" },
    { id: 3, name: "特效", color: "#5B8DEF" },
  ]);

  render(<TagEditor existingTags={[]} recentTags={["UI"]} onApply={vi.fn()} />);

  await userEvent.type(screen.getByPlaceholderText("输入标签..."), "地");

  expect(screen.getByRole("button", { name: "地形" })).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "角色" })).not.toBeInTheDocument();
});

it("does not suggest tags already applied to the current selection", async () => {
  mockedListTags.mockResolvedValue([
    { id: 1, name: "地形", color: "#5B8DEF" },
    { id: 2, name: "角色", color: "#5B8DEF" },
  ]);

  render(<TagEditor existingTags={["地形"]} recentTags={["地形", "角色"]} onApply={vi.fn()} />);

  await screen.findByRole("button", { name: "角色" });

  expect(screen.queryByRole("button", { name: "地形" })).not.toBeInTheDocument();
});

it("applies a suggested tag and clears the input", async () => {
  const onApply = vi.fn();
  mockedListTags.mockResolvedValue([{ id: 1, name: "特效", color: "#5B8DEF" }]);
  render(<TagEditor existingTags={[]} recentTags={[]} onApply={onApply} />);

  await userEvent.type(screen.getByPlaceholderText("输入标签..."), "特");
  await userEvent.click(screen.getByRole("button", { name: "特效" }));

  expect(onApply).toHaveBeenCalledWith("特效");
  expect(screen.getByPlaceholderText("输入标签...")).toHaveValue("");
});
```

- [ ] **Step 3: Run tests to verify failure**

Run:

```powershell
npm test -- src/components/TagEditor.test.tsx
```

Expected: fail until `recentTags` and suggestion behavior are implemented.

- [ ] **Step 4: Update TagEditor props and suggestions**

Change `TagEditor.tsx` props:

```tsx
type Props = {
  existingTags: string[];
  recentTags?: string[];
  onApply: (tagName: string) => void;
};

export function TagEditor({ existingTags, recentTags = [], onApply }: Props) {
```

Replace the current `suggestions` derivation with:

```tsx
const normalizedInput = input.trim().toLowerCase();
const existingSet = new Set(existingTags.map((tag) => tag.toLowerCase()));
const suggestions = [...new Set([...recentTags, ...allTags])]
  .filter((tag) => !existingSet.has(tag.toLowerCase()))
  .filter((tag) => normalizedInput.length === 0 || tag.toLowerCase().includes(normalizedInput))
  .slice(0, 8);
```

Add a helper:

```tsx
function applySuggestion(tag: string) {
  onApply(tag);
  setInput("");
}
```

Use it in suggestion buttons:

```tsx
onClick={() => applySuggestion(tag)}
```

- [ ] **Step 5: Run frontend verification**

Run:

```powershell
npm test -- src/components/TagEditor.test.tsx
npm run build
```

Expected: focused tests pass and production build succeeds.

- [ ] **Step 6: Commit**

```powershell
git add src/components/TagEditor.tsx src/components/TagEditor.test.tsx
git commit -m "feat: add tag suggestions"
```

---

### Task 45: Wire Recent Tags Into App And DetailsPanel

**Files:**
- Modify: `I:\GameResManger\src\App.tsx`
- Modify: `I:\GameResManger\src\components\DetailsPanel.tsx`
- Modify: `I:\GameResManger\src\components\DetailsPanel.test.tsx`

- [ ] **Step 1: Add DetailsPanel prop**

In `DetailsPanel.tsx`, extend props:

```tsx
  recentTags?: string[];
```

In the function signature, default it:

```tsx
  recentTags = [],
```

Pass it to both `TagEditor` instances:

```tsx
<TagEditor existingTags={tags} recentTags={recentTags} onApply={(tagName) => onApplyTag(tagName, ids)} />
```

and:

```tsx
<TagEditor existingTags={tags} recentTags={recentTags} onApply={(tagName) => onApplyTag(tagName, [asset.id])} />
```

- [ ] **Step 2: Add App state and loader**

In `App.tsx`, import `listRecentTags` from `./api/tauri`.

Add state:

```tsx
const [recentTags, setRecentTags] = useState<string[]>([]);
```

Add loader function inside `AppInner`:

```tsx
const loadRecentTags = useCallback(async () => {
  try {
    const tags = await listRecentTags(12);
    setRecentTags(tags.map((tag) => tag.name));
  } catch {
    setRecentTags([]);
  }
}, []);
```

Call it after initial data load succeeds:

```tsx
await loadRecentTags();
```

Include `loadRecentTags` in the `loadData` dependency list.

- [ ] **Step 3: Refresh recent tags after applying a tag**

In `handleApplyTag`, after `await loadData();`, add:

```tsx
await loadRecentTags();
```

This ensures a newly used tag appears at the front of suggestions.

- [ ] **Step 4: Pass recent tags to DetailsPanel**

Add:

```tsx
recentTags={recentTags}
```

to the `DetailsPanel` call in `App.tsx`.

- [ ] **Step 5: Keep DetailsPanel tests compiling**

If `DetailsPanel.test.tsx` creates shared props, add:

```tsx
recentTags={["地形", "特效"]}
```

where useful. If all tests still pass because the prop is optional, do not add unnecessary test noise.

- [ ] **Step 6: Run verification**

Run:

```powershell
npm test -- src/components/DetailsPanel.test.tsx
npm run build
```

Expected: tests pass and build succeeds.

- [ ] **Step 7: Commit**

```powershell
git add src/App.tsx src/components/DetailsPanel.tsx src/components/DetailsPanel.test.tsx
git commit -m "feat: wire recent tags into tag editor"
```

---

### Task 46: Tag Efficiency Smoke QA And Milestone

**Files:**
- Modify: `I:\GameResManger\tests\smoke\README.md`
- Modify: `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`

- [ ] **Step 1: Add smoke checklist**

Append to `tests/smoke/README.md`:

```markdown
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
```

- [ ] **Step 2: Run full verification**

Run:

```powershell
npm test
npm run build
cd I:\GameResManger\src-tauri
cargo test
cargo check
```

Expected: frontend tests pass, frontend build succeeds, Rust tests pass, backend compiles cleanly.

- [ ] **Step 3: Update milestone tracker**

Set `v0.3.2 Tag Efficiency` to `Verified` in the milestone board and add:

```markdown
| 2026-06-05 | v0.3.2 Tag Efficiency | Verified | `npm test`; `npm run build`; `cargo test`; `cargo check` | Recent tags are exposed, TagEditor filters suggestions, and recent suggestions are wired into details and batch tagging. |
```

- [ ] **Step 4: Commit smoke docs**

In `I:\GameResManger`:

```powershell
git add tests/smoke/README.md
git commit -m "docs: add tag efficiency smoke checks"
```

If `G:\oklai999的策划仓库\游戏资源管理器` is a separate Git repository, commit the milestone tracker change separately there.

---

## Acceptance Criteria

- `list_recent_tags` returns tags ordered by `last_used_at DESC, name ASC`.
- Recent tag query clamps the requested limit to `0..50`.
- `TagEditor` shows recent tags before older tag suggestions.
- Typing in the tag input filters suggestions.
- Existing tags for the selected asset or common batch selection are not suggested again.
- Clicking a suggestion applies it and clears the input.
- Single-select and batch-select details panels both receive recent tag suggestions.
- After applying a tag, recent tag suggestions refresh without restarting the app.
- Original source asset files remain unchanged.

## Self-Review

- Spec coverage: Covers v0.3.2 Tag Efficiency from the Sharp Stock roadmap: recent tags, search, suggestions, and batch-friendly tagging.
- Placeholder scan: The plan contains exact files, code snippets, commands, expected results, and commit messages.
- Type consistency: Frontend uses existing `Tag` type from `src/types/asset.ts`; Rust uses existing `crate::models::Tag`.
