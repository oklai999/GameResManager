# Large Library Browsing Performance Implementation Plan

**Progress Sync (2026-06-11):** Completed and verified. Tasks 60-65 delivered paged search, count queries, indexes, and incremental loading.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make large indexed libraries browseable beyond the current 2,000-result cap by adding paginated search responses, compact load-more UI, and database indexes for common filter/sort paths.

**Architecture:** Keep search and file-system access behind Tauri commands. Rust owns SQLite query composition, counting, pagination, and safe bounded limits; React owns page state, reset behavior, and grid rendering. Original source asset folders remain read-only from the app perspective.

**Tech Stack:** Tauri 2, React 18, TypeScript, Vite, Rust, SQLite via `sqlx`, Vitest, cargo test.

---

## Source Context

- Work directory: `I:\GameResManger`
- Product spec: `G:\oklai999的策划仓库\游戏资源管理器\需求整理_v0.1.md`
- Current code state checked on 2026-06-08:
  - Type coverage, recent activity, tag efficiency, path variants, and advanced filters/sorting are implemented in code.
  - `README.md` lists current limitation: search results are capped at 2,000 rows.
  - `src-tauri/src/search.rs` already accepts `limit` and `offset`, but the frontend currently requests one large result window.
  - `src/App.tsx` stores all displayed search results in one array.

## Safety Rules

- Do not delete, move, rename, or modify original asset files.
- Do not write thumbnails, cache, database files, or metadata into source asset folders.
- Do not add delete, batch move, batch rename, AI auto-tagging, atlas export, cloud sync, or full runtime preview features.
- Do not run batch deletion commands such as `del /s`, `rd /s`, `rmdir /s`, `Remove-Item -Recurse`, or `rm -rf`.

## File Structure

- Modify: `I:\GameResManger\src-tauri\src\models.rs`
  - Adds `AssetSearchResponse` for paginated results plus total count.
- Modify: `I:\GameResManger\src-tauri\src\search.rs`
  - Reuses search conditions for both row query and count query; keeps sort whitelist and bind parameters.
- Modify: `I:\GameResManger\src-tauri\src\commands.rs`
  - Adds `search_assets_page` without removing the existing `search_assets` command.
- Modify: `I:\GameResManger\src-tauri\src\lib.rs`
  - Registers the new command.
- Create: `I:\GameResManger\src-tauri\migrations\0006_search_performance_indexes.sql`
  - Adds indexes for frequent browse/filter/sort fields.
- Modify: `I:\GameResManger\src\types\asset.ts`
  - Mirrors `AssetSearchResponse`.
- Modify: `I:\GameResManger\src\api\tauri.ts`
  - Adds typed `searchAssetsPage`.
- Modify: `I:\GameResManger\src\App.tsx`
  - Uses paged search state, resets pages when query/filter/sort changes, and appends results on load-more.
- Modify: `I:\GameResManger\src\components\AssetGrid.tsx`
  - Remains a presentational grid; no filesystem logic added.
- Modify: `I:\GameResManger\tests\smoke\README.md`
  - Adds large-library pagination smoke checks.
- Modify: `I:\GameResManger\README.md`
  - Updates the limitation note after pagination is verified.

---

## Task Status Board

| Done | Task | Owner | Scope | Required Verification | Notes |
|------|------|-------|-------|-----------------------|-------|
| [x] | Task 60 | Agent | Sync current roadmap status | Review plan docs and milestone tracker | Advanced filters/sorting plan board and Sharp Stock milestone tracker synchronized. |
| [x] | Task 61 | Agent | Add paginated search response contract | `cargo check`; `npm run build` | `AssetSearchResponse` added to Rust/TypeScript and `searchAssetsPage` wrapper added. |
| [x] | Task 62 | Agent | Implement backend count + page query | `cargo test search`; `cargo check` | `search_assets_page` implemented and registered; count query uses same filters and bind order as row query. |
| [x] | Task 63 | Agent | Add search performance indexes | `cargo test`; `cargo check` | Added `0006_search_performance_indexes.sql` with 9 search/browse indexes. |
| [x] | Task 64 | Agent | Wire frontend paged loading | `npm test`; `npm run build` | Frontend now loads first page and appends additional pages with `加载更多`. |
| [x] | Task 65 | Agent | Add smoke QA and final verification | `npm test`; `npm run build`; `cargo test`; `cargo check` | Automated verification passed; GUI large-library smoke remains manual/user-run. |

---

### Task 60: Sync Current Roadmap Status

**Files:**
- Modify: `I:\GameResManger\docs\superpowers\plans\2026-06-08-advanced-filters-and-sorting.md`
- Modify: `G:\oklai999的策划仓库\游戏资源管理器\里程碑记录\2026-06-04-sharp-stock-reference-milestones.md`

- [x] **Step 1: Confirm current code contains v0.5 search features**

Run:

```powershell
Select-String -Path 'I:\GameResManger\src\App.tsx','I:\GameResManger\src-tauri\src\search.rs','I:\GameResManger\src\components\SearchToolbar.tsx' -Pattern 'min_file_size|sort_by|advanced-filter-row'
```

Expected: output shows advanced filter and sorting fields in App, Rust search, and SearchToolbar.

- [x] **Step 2: Update the advanced filters plan board**

In `docs/superpowers/plans/2026-06-08-advanced-filters-and-sorting.md`, change Tasks 54-59 from `[ ]` to `[x]` only after matching implementation and smoke checklist are confirmed in the current code.

- [x] **Step 3: Add milestone entry**

Append to the milestone tracker completion log:

```markdown
| 2026-06-08 | v0.5 Advanced Filters And Sorting | Verified | `npm test`; `npm run build`; `cargo test`; `cargo check` | Size, dimensions, modified time filters, and whitelisted sorting are implemented. |
```

- [x] **Step 4: Review status**

Run:

```powershell
git status --short
```

Expected: only planning document changes are present unless previous user work already exists.

---

### Task 61: Add Paginated Search Contract

**Files:**
- Modify: `I:\GameResManger\src-tauri\src\models.rs`
- Modify: `I:\GameResManger\src\types\asset.ts`
- Modify: `I:\GameResManger\src\api\tauri.ts`

- [x] **Step 1: Add Rust response model**

Add to `models.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSearchResponse {
    pub assets: Vec<Asset>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}
```

- [x] **Step 2: Add TypeScript response type**

Add to `asset.ts`:

```ts
export type AssetSearchResponse = {
  assets: Asset[];
  total_count: number;
  limit: number;
  offset: number;
};
```

- [x] **Step 3: Add API wrapper**

In `src/api/tauri.ts`, import `AssetSearchResponse` and add:

```ts
export async function searchAssetsPage(req: AssetSearchRequest): Promise<AssetSearchResponse> {
  return invoke<AssetSearchResponse>("search_assets_page", { req });
}
```

- [x] **Step 4: Run contract checks**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo check
cd I:\GameResManger
npm run build
```

Expected: checks may fail until Task 62 registers the new command if the wrapper is imported immediately. Do not wire the wrapper into App in this task.

---

### Task 62: Implement Backend Count And Page Query

**Files:**
- Modify: `I:\GameResManger\src-tauri\src\search.rs`
- Modify: `I:\GameResManger\src-tauri\src\commands.rs`
- Modify: `I:\GameResManger\src-tauri\src\lib.rs`

- [x] **Step 1: Add backend tests**

In `search.rs`, add:

```rust
#[tokio::test]
async fn paged_search_returns_total_count() {
    let pool = search_test_pool().await;
    let mut req = empty_request();
    req.limit = 2;
    req.offset = 0;

    let page = search_assets_page(&pool, &req).await.unwrap();

    assert_eq!(page.assets.len(), 2);
    assert_eq!(page.total_count, 3);
    assert_eq!(page.limit, 2);
    assert_eq!(page.offset, 0);
}

#[tokio::test]
async fn paged_search_applies_same_filters_to_count() {
    let pool = search_test_pool().await;
    let mut req = empty_request();
    req.min_file_size = Some(200);
    req.max_file_size = Some(900);
    req.limit = 20;
    req.offset = 0;

    let page = search_assets_page(&pool, &req).await.unwrap();

    assert_eq!(page.assets.len(), 1);
    assert_eq!(page.total_count, 1);
    assert_eq!(page.assets[0].file_name, "mid.png");
}
```

- [x] **Step 2: Run tests to verify failure**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo test search::tests::paged_search
```

Expected: fail until `search_assets_page` exists.

- [x] **Step 3: Implement paged search**

Add `search_assets_page`:

```rust
pub async fn search_assets_page(
    db: &SqlitePool,
    req: &AssetSearchRequest,
) -> anyhow::Result<crate::models::AssetSearchResponse> {
    let assets = search_assets(db, req).await?;
    let total_count = count_search_assets(db, req).await?;
    let limit = req.limit.clamp(1, 500);
    let offset = req.offset.max(0);
    Ok(crate::models::AssetSearchResponse {
        assets,
        total_count,
        limit,
        offset,
    })
}
```

Implement `count_search_assets` using the same conditions and bind order as `search_assets`, but with:

```sql
SELECT COUNT(*) FROM assets
```

Do not include `ORDER BY`, `LIMIT`, or `OFFSET` in the count SQL.

- [x] **Step 4: Add Tauri command**

In `commands.rs`:

```rust
#[tauri::command]
pub async fn search_assets_page(
    db: State<'_, SqlitePool>,
    req: crate::models::AssetSearchRequest,
) -> Result<crate::models::AssetSearchResponse, CommandError> {
    search::search_assets_page(&*db, &req).await.map_err(Into::into)
}
```

Register it in `lib.rs`.

- [x] **Step 5: Run backend verification**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo test search
cargo check
```

Expected: all search tests pass and backend compiles.

---

### Task 63: Add Search Performance Indexes

**Files:**
- Create: `I:\GameResManger\src-tauri\migrations\0006_search_performance_indexes.sql`

- [x] **Step 1: Add migration**

Create:

```sql
CREATE INDEX IF NOT EXISTS idx_assets_file_size ON assets(file_size);
CREATE INDEX IF NOT EXISTS idx_assets_modified_at ON assets(modified_at);
CREATE INDEX IF NOT EXISTS idx_assets_dimensions ON assets(width, height);
CREATE INDEX IF NOT EXISTS idx_assets_type_name ON assets(asset_type, file_name);
CREATE INDEX IF NOT EXISTS idx_assets_folder_name ON assets(library_folder_id, file_name);
CREATE INDEX IF NOT EXISTS idx_assets_favorite_name ON assets(is_favorite, file_name);
CREATE INDEX IF NOT EXISTS idx_assets_missing_name ON assets(is_missing, file_name);
CREATE INDEX IF NOT EXISTS idx_collection_assets_collection_asset ON collection_assets(collection_id, asset_id);
CREATE INDEX IF NOT EXISTS idx_asset_tags_tag_asset ON asset_tags(tag_id, asset_id);
```

- [x] **Step 2: Run migration-aware checks**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo test
cargo check
```

Expected: migrations compile and tests pass.

---

### Task 64: Wire Frontend Paged Loading

**Files:**
- Modify: `I:\GameResManger\src\App.tsx`
- Modify: `I:\GameResManger\src\components\AssetGrid.tsx`
- Modify: `I:\GameResManger\src\styles.css`

- [x] **Step 1: Add paging constants and state**

In `App.tsx`, replace the single large result limit with:

```ts
const SEARCH_PAGE_SIZE = 200;
```

Add state:

```ts
const [totalCount, setTotalCount] = useState(0);
const [isLoadingMore, setIsLoadingMore] = useState(false);
```

- [x] **Step 2: Use paged API for first page**

In `executeSearch`, call:

```ts
const page = await searchAssetsPage({ ...req, limit: SEARCH_PAGE_SIZE, offset: 0 });
setAssets(mergeAssetTags(page.assets, assetTags));
setTotalCount(page.total_count);
```

Use the existing tag merge pattern from `App.tsx`; do not duplicate tag rendering logic in `AssetGrid`.

- [x] **Step 3: Add load-more handler**

Add:

```ts
const handleLoadMore = useCallback(async () => {
  if (isLoadingMore || assets.length >= totalCount) return;
  setIsLoadingMore(true);
  try {
    const req = buildCurrentSearchRequest(assets.length);
    const page = await searchAssetsPage(req);
    setAssets((prev) => [...prev, ...mergeAssetTags(page.assets, assetTags)]);
    setTotalCount(page.total_count);
  } catch (e) {
    showError(String(e));
  } finally {
    setIsLoadingMore(false);
  }
}, [isLoadingMore, assets.length, totalCount, assetTags, showError]);
```

If `buildCurrentSearchRequest` does not exist yet, extract the request-building code from `executeSearch` into a local callback that accepts `offset`.

- [x] **Step 4: Render result count and load-more button**

Below the grid in `App.tsx`, render:

```tsx
<div className="result-footer">
  <span>已显示 {displayAssets.length} / {totalCount}</span>
  {displayAssets.length < totalCount && (
    <button onClick={handleLoadMore} disabled={isLoadingMore}>
      {isLoadingMore ? "加载中..." : "加载更多"}
    </button>
  )}
</div>
```

- [x] **Step 5: Style footer**

Add:

```css
.result-footer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 14px 0 4px;
  color: #8f9bb3;
  font-size: 13px;
}

.result-footer button {
  height: 32px;
  border: 1px solid #2d3447;
  border-radius: 6px;
  background: #1d2431;
  color: #dce5f5;
  padding: 0 12px;
  cursor: pointer;
}
```

- [x] **Step 6: Run frontend verification**

Run:

```powershell
npm test
npm run build
```

Expected: all frontend tests pass and production build succeeds.

---

### Task 65: Smoke QA And Final Verification

**Files:**
- Modify: `I:\GameResManger\tests\smoke\README.md`
- Modify: `I:\GameResManger\README.md`

- [x] **Step 1: Add smoke checklist**

Append to `tests/smoke/README.md`:

```markdown
## Large Library Pagination Smoke Test

- Scan or choose a library with more than 250 indexed assets.
- Confirm the first page appears without waiting for all matching assets to render.
- Confirm the footer shows `已显示 200 / 总数` or the current page size equivalent.
- Click `加载更多` and confirm additional cards append without clearing selection unexpectedly.
- Change a search query, filter, or sort option and confirm results reset to the first page.
- Confirm favorite, tag, collection, details, open file, open folder, and copy path still work on loaded pages.
- Confirm no source file is deleted, moved, renamed, modified, or written to during this smoke test.
```

- [x] **Step 2: Update README limitation**

Change:

```markdown
- Search results are capped at 2,000 rows.
```

to:

```markdown
- Search results are loaded in pages; very large libraries may still need full grid virtualization and SQLite FTS.
```

- [x] **Step 3: Run final automated verification**

Run:

```powershell
npm test
npm run build
cd I:\GameResManger\src-tauri
cargo test
cargo check
```

Expected:

- All frontend tests pass.
- Production build succeeds.
- All Rust tests pass.
- Backend compiles.

- [x] **Step 4: Review working tree scope**

Run:

```powershell
git status --short
```

Expected: changes are limited to this plan's files plus any pre-existing unrelated worktree changes. Do not stage or revert unrelated files.

---

## Acceptance Criteria

- Search results load in bounded pages instead of one 2,000-row request.
- Users can load additional result pages from the workbench.
- Changing query, scope, filter, folder, collection, recent/favorite/missing/type filter, or sort resets to the first page.
- Backend count and row queries use the same search filters.
- Backend still uses bind parameters for user-provided values.
- Backend sorting still uses a whitelist for column names and direction.
- Common filter/sort paths have SQLite indexes.
- Existing safe asset actions still work on paged results.
- Original source asset files remain untouched.

## Self-Review

- Spec coverage: This plan supports the core "large local library" scenario by improving browsing and filtering after indexing large folders.
- Placeholder scan: The plan contains concrete file paths, code snippets, commands, expected results, and acceptance criteria.
- Type consistency: `AssetSearchResponse` is named the same in Rust and TypeScript, and `search_assets_page` maps to frontend `searchAssetsPage`.
- Scope check: This phase does not add AI tagging, atlas tools, cloud sync, delete, move, rename, or full runtime previews.
