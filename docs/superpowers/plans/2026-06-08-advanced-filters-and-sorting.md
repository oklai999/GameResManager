# Advanced Filters And Sorting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-class file size, dimension, modified-time filters and stable sorting controls so users can narrow large local asset libraries more precisely.

**Architecture:** Extend the existing Tauri search contract instead of adding a separate filtering layer. React owns form state and sends typed `AssetSearchRequest` values to Rust; Rust owns SQL construction with whitelisted sort columns and bound filter parameters.

**Tech Stack:** Tauri 2, React 18, TypeScript, Vite, Rust, SQLite via `sqlx`, Vitest, cargo test.

---

## Source Context

- Work directory: `I:\GameResManger`
- Product spec: `G:\oklai999的策划仓库\游戏资源管理器\需求整理_v0.1.md`
- Relevant spec sections:
  - `4.2 中间资源工作区`: filter by type, dimensions, file size, modified time, favorite, missing; sort by name, size, modified time, type.
  - `14.2 对现有需求的强化`: filtering should cover type, favorite, collection, missing state, dimensions, file size, modified time.
- Current state checked on 2026-06-08:
  - Search scope exists in `src/components/SearchToolbar.tsx`.
  - `AssetSearchRequest` already supports query fields, type, folder, collection, favorite, missing, limit, offset.
  - `search.rs` always sorts by `file_name`.
  - Size, dimensions, modified-time range filters and selectable sort order are not yet implemented as an end-to-end contract.

## Safety Rules

- Do not delete, move, rename, or modify original asset files.
- Do not write cache, database, metadata, or generated files into source asset folders.
- Do not add delete, batch move, batch rename, AI auto-tagging, atlas export, cloud sync, or full runtime preview features.
- Do not run batch deletion commands such as `del /s`, `rd /s`, `rmdir /s`, `Remove-Item -Recurse`, or `rm -rf`.
- Existing unrelated deleted files and release artifacts in the working tree must not be reverted, staged, or cleaned.

## File Structure

- Modify: `I:\GameResManger\src-tauri\src\models.rs`
  - Extends `AssetSearchRequest` with numeric/date filters and sort fields.
- Modify: `I:\GameResManger\src-tauri\src\search.rs`
  - Adds SQL conditions for file size, dimensions, modified time, and whitelisted ordering.
- Modify: `I:\GameResManger\src\types\asset.ts`
  - Mirrors new search request fields and adds frontend filter/sort helper types.
- Modify: `I:\GameResManger\src\components\SearchToolbar.tsx`
  - Adds compact filter and sort controls above the grid.
- Modify: `I:\GameResManger\src\components\SearchToolbar.test.tsx`
  - Covers filter input, reset, and sort callbacks.
- Modify: `I:\GameResManger\src\App.tsx`
  - Stores filter/sort state and sends it to `searchAssets`.
- Modify: `I:\GameResManger\tests\smoke\README.md`
  - Adds manual smoke checks for size, dimension, modified-time, and sorting behavior.

---

## Task Status Board

| Done | Task | Owner | Scope | Required Verification | Notes |
|------|------|-------|-------|-----------------------|-------|
| [x] | Task 54 | Agent | Extend search contract types | `cargo check`; `npm run build` | Adds request fields only; no UI yet. |
| [x] | Task 55 | Agent | Implement backend range filters and sorting | `cargo test search`; `cargo check` | Uses bound parameters and whitelisted sort columns. |
| [x] | Task 56 | Agent | Add frontend filter/sort state types | `npm run build` | Keeps TypeScript contract aligned with Rust. |
| [x] | Task 57 | Agent | Add SearchToolbar filter and sort controls | `npm test -- src/components/SearchToolbar.test.tsx`; `npm run build` | Compact tool UI, no landing-page style. |
| [x] | Task 58 | Agent | Wire filters and sorting into App search | `npm test`; `npm run build` | Search request carries filters/sort to Tauri. |
| [x] | Task 59 | Agent | Add smoke QA and final verification | `npm test`; `npm run build`; `cargo test`; `cargo check` | Documentation and final checks. |

---

### Task 54: Extend Search Contract Types

**Files:**
- Modify: `I:\GameResManger\src-tauri\src\models.rs`
- Modify: `I:\GameResManger\src\types\asset.ts`

- [ ] **Step 1: Add Rust request fields**

In `src-tauri/src/models.rs`, replace `AssetSearchRequest` with:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSearchRequest {
    pub query: String,
    pub search_file_name: bool,
    pub search_note: bool,
    pub search_path: bool,
    pub search_tags: bool,
    pub asset_type: Option<String>,
    pub library_folder_id: Option<i64>,
    pub collection_id: Option<i64>,
    pub is_favorite: Option<bool>,
    pub is_missing: Option<bool>,
    pub min_file_size: Option<i64>,
    pub max_file_size: Option<i64>,
    pub min_width: Option<i64>,
    pub max_width: Option<i64>,
    pub min_height: Option<i64>,
    pub max_height: Option<i64>,
    pub modified_after: Option<String>,
    pub modified_before: Option<String>,
    pub sort_by: String,
    pub sort_direction: String,
    pub limit: i64,
    pub offset: i64,
}
```

- [ ] **Step 2: Add TypeScript search helper types**

In `src/types/asset.ts`, replace `AssetSearchRequest` with:

```ts
export type AssetSortBy = "file_name" | "file_size" | "modified_at" | "asset_type";

export type SortDirection = "asc" | "desc";

export type AssetSearchFilters = {
  min_file_size: number | null;
  max_file_size: number | null;
  min_width: number | null;
  max_width: number | null;
  min_height: number | null;
  max_height: number | null;
  modified_after: string | null;
  modified_before: string | null;
};

export type AssetSearchSort = {
  sort_by: AssetSortBy;
  sort_direction: SortDirection;
};

export type AssetSearchRequest = {
  query: string;
  search_file_name: boolean;
  search_note: boolean;
  search_path: boolean;
  search_tags: boolean;
  asset_type: string | null;
  library_folder_id: number | null;
  collection_id: number | null;
  is_favorite: boolean | null;
  is_missing: boolean | null;
  min_file_size: number | null;
  max_file_size: number | null;
  min_width: number | null;
  max_width: number | null;
  min_height: number | null;
  max_height: number | null;
  modified_after: string | null;
  modified_before: string | null;
  sort_by: AssetSortBy;
  sort_direction: SortDirection;
  limit: number;
  offset: number;
};
```

- [ ] **Step 3: Update Rust test helper to compile**

In `src-tauri/src/search.rs`, update `empty_request()` so every new field is set:

```rust
fn empty_request() -> AssetSearchRequest {
    AssetSearchRequest {
        query: String::new(),
        search_file_name: true,
        search_note: false,
        search_path: false,
        search_tags: false,
        asset_type: None,
        library_folder_id: None,
        collection_id: None,
        is_favorite: None,
        is_missing: None,
        min_file_size: None,
        max_file_size: None,
        min_width: None,
        max_width: None,
        min_height: None,
        max_height: None,
        modified_after: None,
        modified_before: None,
        sort_by: "file_name".to_string(),
        sort_direction: "asc".to_string(),
        limit: 200,
        offset: 0,
    }
}
```

- [ ] **Step 4: Run contract checks**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo check
cd I:\GameResManger
npm run build
```

Expected: Rust and TypeScript may fail until all request construction sites are updated in later tasks. If they fail only because new fields are missing in `App.tsx` or search tests, proceed to the next task before finalizing this slice.

- [ ] **Step 5: Commit**

After Task 55 and Task 56 make both compilers pass, commit this task together with those dependent changes if needed:

```powershell
git add src-tauri/src/models.rs src/types/asset.ts
git commit -m "feat: extend asset search contract"
```

---

### Task 55: Implement Backend Range Filters And Sorting

**Files:**
- Modify: `I:\GameResManger\src-tauri\src\search.rs`

- [ ] **Step 1: Add focused backend tests**

Add these tests inside the existing `#[cfg(test)] mod tests` in `src-tauri/src/search.rs`:

```rust
#[tokio::test]
async fn filters_by_file_size_dimensions_and_modified_time() {
    let pool = search_test_pool().await;

    let mut req = empty_request();
    req.min_file_size = Some(200);
    req.max_file_size = Some(900);
    req.min_width = Some(64);
    req.max_width = Some(512);
    req.min_height = Some(64);
    req.max_height = Some(512);
    req.modified_after = Some("2024-01-02T00:00:00Z".to_string());
    req.modified_before = Some("2024-01-05T00:00:00Z".to_string());

    let results = search_assets(&pool, &req).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].file_name, "mid.png");
}

#[tokio::test]
async fn sorts_by_size_descending() {
    let pool = search_test_pool().await;

    let mut req = empty_request();
    req.sort_by = "file_size".to_string();
    req.sort_direction = "desc".to_string();

    let results = search_assets(&pool, &req).await.unwrap();
    let names: Vec<String> = results.into_iter().map(|asset| asset.file_name).collect();

    assert_eq!(names, vec!["large.png", "mid.png", "small.png"]);
}

#[tokio::test]
async fn invalid_sort_values_fall_back_to_file_name_ascending() {
    let pool = search_test_pool().await;

    let mut req = empty_request();
    req.sort_by = "absolute_path; DROP TABLE assets".to_string();
    req.sort_direction = "sideways".to_string();

    let results = search_assets(&pool, &req).await.unwrap();
    let names: Vec<String> = results.into_iter().map(|asset| asset.file_name).collect();

    assert_eq!(names, vec!["large.png", "mid.png", "small.png"]);
}
```

- [ ] **Step 2: Add test fixture helper**

Add this helper in the same test module:

```rust
async fn search_test_pool() -> SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query(
        "CREATE TABLE assets (
            id INTEGER PRIMARY KEY,
            library_folder_id INTEGER NOT NULL,
            absolute_path TEXT NOT NULL UNIQUE,
            file_name TEXT NOT NULL,
            extension TEXT NOT NULL,
            asset_type TEXT NOT NULL,
            file_size INTEGER NOT NULL DEFAULT 0,
            modified_at TEXT NOT NULL,
            width INTEGER,
            height INTEGER,
            thumbnail_path TEXT,
            thumbnail_status TEXT NOT NULL DEFAULT 'none',
            thumbnail_error TEXT,
            note TEXT NOT NULL DEFAULT '',
            is_favorite INTEGER NOT NULL DEFAULT 0,
            is_missing INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TABLE tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#5B8DEF',
            created_at TEXT NOT NULL,
            last_used_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TABLE asset_tags (
            asset_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (asset_id, tag_id)
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TABLE collection_assets (
            collection_id INTEGER NOT NULL,
            asset_id INTEGER NOT NULL,
            PRIMARY KEY (collection_id, asset_id)
        )",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO assets
        (id, library_folder_id, absolute_path, file_name, extension, asset_type, file_size, modified_at, width, height, note, created_at, updated_at)
        VALUES
        (1, 1, '/test/small.png', 'small.png', 'png', 'image', 100, '2024-01-01T00:00:00Z', 32, 32, '', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
        (2, 1, '/test/mid.png', 'mid.png', 'png', 'image', 500, '2024-01-03T00:00:00Z', 128, 128, '', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z'),
        (3, 1, '/test/large.png', 'large.png', 'png', 'image', 2000, '2024-01-06T00:00:00Z', 2048, 2048, '', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}
```

- [ ] **Step 3: Run tests to verify failure**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo test search::tests
```

Expected: fail until filters and sorting are implemented.

- [ ] **Step 4: Add backend filter conditions**

In `search_assets`, after existing favorite/missing conditions, add:

```rust
if req.min_file_size.is_some() { conditions.push("file_size >= ?".to_string()); }
if req.max_file_size.is_some() { conditions.push("file_size <= ?".to_string()); }
if req.min_width.is_some() { conditions.push("width >= ?".to_string()); }
if req.max_width.is_some() { conditions.push("width <= ?".to_string()); }
if req.min_height.is_some() { conditions.push("height >= ?".to_string()); }
if req.max_height.is_some() { conditions.push("height <= ?".to_string()); }
if req.modified_after.is_some() { conditions.push("modified_at >= ?".to_string()); }
if req.modified_before.is_some() { conditions.push("modified_at <= ?".to_string()); }
```

After existing binds for `is_missing`, add:

```rust
if let Some(v) = req.min_file_size { query = query.bind(v); }
if let Some(v) = req.max_file_size { query = query.bind(v); }
if let Some(v) = req.min_width { query = query.bind(v); }
if let Some(v) = req.max_width { query = query.bind(v); }
if let Some(v) = req.min_height { query = query.bind(v); }
if let Some(v) = req.max_height { query = query.bind(v); }
if let Some(ref v) = req.modified_after { query = query.bind(v); }
if let Some(ref v) = req.modified_before { query = query.bind(v); }
```

- [ ] **Step 5: Replace fixed ordering with whitelisted ordering**

Replace:

```rust
sql.push_str(" ORDER BY file_name LIMIT ? OFFSET ?");
```

with:

```rust
let sort_column = match req.sort_by.as_str() {
    "file_size" => "file_size",
    "modified_at" => "modified_at",
    "asset_type" => "asset_type",
    _ => "file_name",
};
let sort_direction = if req.sort_direction.eq_ignore_ascii_case("desc") {
    "DESC"
} else {
    "ASC"
};
sql.push_str(&format!(
    " ORDER BY {} {}, file_name ASC LIMIT ? OFFSET ?",
    sort_column, sort_direction
));
```

- [ ] **Step 6: Update existing integration request literals**

Every `AssetSearchRequest { ... }` literal in `search.rs` tests must include:

```rust
min_file_size: None,
max_file_size: None,
min_width: None,
max_width: None,
min_height: None,
max_height: None,
modified_after: None,
modified_before: None,
sort_by: "file_name".to_string(),
sort_direction: "asc".to_string(),
```

- [ ] **Step 7: Run backend verification**

Run:

```powershell
cd I:\GameResManger\src-tauri
cargo test search
cargo check
```

Expected: all search tests pass and backend compiles cleanly.

- [ ] **Step 8: Commit**

```powershell
git add src-tauri/src/models.rs src-tauri/src/search.rs
git commit -m "feat: add asset search filters and sorting"
```

---

### Task 56: Add Frontend Filter And Sort State Types

**Files:**
- Modify: `I:\GameResManger\src\types\asset.ts`
- Modify: `I:\GameResManger\src\App.tsx`

- [ ] **Step 1: Add default state helpers in App**

In `src/App.tsx`, update the type import:

```ts
import type {
  Asset,
  AssetSearchFilters,
  AssetSearchRequest,
  AssetSearchSort,
  Collection,
  FolderAssetCounts,
  LibraryFolder,
  ScanJob,
  ScanSettings,
  SearchScope,
} from "./types/asset";
```

Add constants near `SEARCH_RESULT_LIMIT`:

```ts
const DEFAULT_SEARCH_FILTERS: AssetSearchFilters = {
  min_file_size: null,
  max_file_size: null,
  min_width: null,
  max_width: null,
  min_height: null,
  max_height: null,
  modified_after: null,
  modified_before: null,
};

const DEFAULT_SEARCH_SORT: AssetSearchSort = {
  sort_by: "file_name",
  sort_direction: "asc",
};
```

- [ ] **Step 2: Add state in AppInner**

Add state next to `query` and `scope`:

```ts
const [filters, setFilters] = useState<AssetSearchFilters>(DEFAULT_SEARCH_FILTERS);
const [sort, setSort] = useState<AssetSearchSort>(DEFAULT_SEARCH_SORT);
```

- [ ] **Step 3: Include fields in search request**

In `executeSearch`, add these fields to `req`:

```ts
min_file_size: filters.min_file_size,
max_file_size: filters.max_file_size,
min_width: filters.min_width,
max_width: filters.max_width,
min_height: filters.min_height,
max_height: filters.max_height,
modified_after: filters.modified_after,
modified_before: filters.modified_before,
sort_by: sort.sort_by,
sort_direction: sort.sort_direction,
```

Add `filters` and `sort` to the `executeSearch` dependency list.

- [ ] **Step 4: Reset selection when filters or sort change**

Add callbacks:

```ts
const handleFiltersChange = useCallback((next: AssetSearchFilters) => {
  setFilters(next);
  setSelectedIds([]);
}, []);

const handleSortChange = useCallback((next: AssetSearchSort) => {
  setSort(next);
  setSelectedIds([]);
}, []);
```

- [ ] **Step 5: Run TypeScript build**

Run:

```powershell
npm run build
```

Expected: build may fail until `SearchToolbar` accepts the new props in Task 57. Continue to Task 57 before finalizing.

---

### Task 57: Add SearchToolbar Filter And Sort Controls

**Files:**
- Modify: `I:\GameResManger\src\components\SearchToolbar.tsx`
- Modify: `I:\GameResManger\src\components\SearchToolbar.test.tsx`
- Modify: `I:\GameResManger\src\styles.css`

- [ ] **Step 1: Update SearchToolbar props**

In `SearchToolbar.tsx`, update imports:

```ts
import { RotateCcw, Search } from "lucide-react";
import type { AssetSearchFilters, AssetSearchSort, SearchScope } from "../types/asset";
```

Replace `Props` with:

```ts
type Props = {
  query: string;
  scope: SearchScope;
  filters: AssetSearchFilters;
  sort: AssetSearchSort;
  onQueryChange: (query: string) => void;
  onScopeChange: (scope: SearchScope) => void;
  onFiltersChange: (filters: AssetSearchFilters) => void;
  onSortChange: (sort: AssetSearchSort) => void;
};
```

- [ ] **Step 2: Add toolbar helper functions**

Inside `SearchToolbar`, add:

```ts
function parseNumber(value: string): number | null {
  const trimmed = value.trim();
  if (trimmed.length === 0) return null;
  const parsed = Number(trimmed);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : null;
}

function updateNumberFilter(key: keyof AssetSearchFilters, value: string) {
  onFiltersChange({ ...filters, [key]: parseNumber(value) });
}

function updateKilobyteFilter(key: "min_file_size" | "max_file_size", value: string) {
  const parsed = parseNumber(value);
  onFiltersChange({ ...filters, [key]: parsed == null ? null : parsed * 1024 });
}

function updateDateFilter(key: "modified_after" | "modified_before", value: string) {
  onFiltersChange({ ...filters, [key]: value ? `${value}T00:00:00Z` : null });
}

function dateInputValue(value: string | null): string {
  return value ? value.slice(0, 10) : "";
}

function resetAdvancedFilters() {
  onFiltersChange({
    min_file_size: null,
    max_file_size: null,
    min_width: null,
    max_width: null,
    min_height: null,
    max_height: null,
    modified_after: null,
    modified_before: null,
  });
}
```

- [ ] **Step 3: Add failing tests**

In `SearchToolbar.test.tsx`, update imports:

```tsx
import { fireEvent, render, screen } from "@testing-library/react";
```

Add defaults:

```ts
const filters = {
  min_file_size: null,
  max_file_size: null,
  min_width: null,
  max_width: null,
  min_height: null,
  max_height: null,
  modified_after: null,
  modified_before: null,
};

const sort = {
  sort_by: "file_name" as const,
  sort_direction: "asc" as const,
};
```

Add tests:

```tsx
it("updates file size filters", async () => {
  const onFiltersChange = vi.fn();
  render(
    <SearchToolbar
      query=""
      scope={scope}
      filters={filters}
      sort={sort}
      onQueryChange={vi.fn()}
      onScopeChange={vi.fn()}
      onFiltersChange={onFiltersChange}
      onSortChange={vi.fn()}
    />
  );

  fireEvent.change(screen.getByLabelText("最小大小 KB"), { target: { value: "128" } });

  expect(onFiltersChange).toHaveBeenCalledWith({ ...filters, min_file_size: 128 * 1024 });
});

it("updates sort controls", async () => {
  const onSortChange = vi.fn();
  render(
    <SearchToolbar
      query=""
      scope={scope}
      filters={filters}
      sort={sort}
      onQueryChange={vi.fn()}
      onScopeChange={vi.fn()}
      onFiltersChange={vi.fn()}
      onSortChange={onSortChange}
    />
  );

  await userEvent.selectOptions(screen.getByLabelText("排序字段"), "file_size");
  await userEvent.selectOptions(screen.getByLabelText("排序方向"), "desc");

  expect(onSortChange).toHaveBeenCalledWith({ ...sort, sort_by: "file_size" });
  expect(onSortChange).toHaveBeenCalledWith({ ...sort, sort_direction: "desc" });
});

it("resets advanced filters", async () => {
  const onFiltersChange = vi.fn();
  render(
    <SearchToolbar
      query=""
      scope={scope}
      filters={{ ...filters, min_width: 64, max_width: 256 }}
      sort={sort}
      onQueryChange={vi.fn()}
      onScopeChange={vi.fn()}
      onFiltersChange={onFiltersChange}
      onSortChange={vi.fn()}
    />
  );

  await userEvent.click(screen.getByRole("button", { name: "重置高级筛选" }));

  expect(onFiltersChange).toHaveBeenCalledWith(filters);
});
```

- [ ] **Step 4: Run tests to verify failure**

Run:

```powershell
npm test -- src/components/SearchToolbar.test.tsx
```

Expected: fail until controls are implemented.

- [ ] **Step 5: Render compact controls**

Below the existing scope segments in `SearchToolbar.tsx`, add:

```tsx
<div className="advanced-filter-row">
  <label>
    <span>大小 KB</span>
    <input
      type="number"
      min="0"
      aria-label="最小大小 KB"
      value={filters.min_file_size == null ? "" : Math.round(filters.min_file_size / 1024)}
      onChange={(event) => updateKilobyteFilter("min_file_size", event.target.value)}
      placeholder="最小"
    />
  </label>
  <label>
    <span>到</span>
    <input
      type="number"
      min="0"
      aria-label="最大大小 KB"
      value={filters.max_file_size == null ? "" : Math.round(filters.max_file_size / 1024)}
      onChange={(event) => updateKilobyteFilter("max_file_size", event.target.value)}
      placeholder="最大"
    />
  </label>
  <label>
    <span>宽</span>
    <input
      type="number"
      min="0"
      aria-label="最小宽度"
      value={filters.min_width ?? ""}
      onChange={(event) => updateNumberFilter("min_width", event.target.value)}
      placeholder="最小"
    />
  </label>
  <label>
    <span>高</span>
    <input
      type="number"
      min="0"
      aria-label="最小高度"
      value={filters.min_height ?? ""}
      onChange={(event) => updateNumberFilter("min_height", event.target.value)}
      placeholder="最小"
    />
  </label>
  <label>
    <span>修改后</span>
    <input
      type="date"
      aria-label="修改日期从"
      value={dateInputValue(filters.modified_after)}
      onChange={(event) => updateDateFilter("modified_after", event.target.value)}
    />
  </label>
  <label>
    <span>修改前</span>
    <input
      type="date"
      aria-label="修改日期到"
      value={dateInputValue(filters.modified_before)}
      onChange={(event) => updateDateFilter("modified_before", event.target.value)}
    />
  </label>
  <label>
    <span>排序</span>
    <select
      aria-label="排序字段"
      value={sort.sort_by}
      onChange={(event) => onSortChange({ ...sort, sort_by: event.target.value as AssetSearchSort["sort_by"] })}
    >
      <option value="file_name">名称</option>
      <option value="file_size">大小</option>
      <option value="modified_at">修改时间</option>
      <option value="asset_type">类型</option>
    </select>
  </label>
  <label>
    <span>方向</span>
    <select
      aria-label="排序方向"
      value={sort.sort_direction}
      onChange={(event) => onSortChange({ ...sort, sort_direction: event.target.value as AssetSearchSort["sort_direction"] })}
    >
      <option value="asc">升序</option>
      <option value="desc">降序</option>
    </select>
  </label>
  <button type="button" className="icon-text-btn" onClick={resetAdvancedFilters} aria-label="重置高级筛选">
    <RotateCcw size={14} aria-hidden="true" />
    重置
  </button>
</div>
```

- [ ] **Step 6: Style controls**

Add to `src/styles.css`:

```css
.search-toolbar {
  flex-wrap: wrap;
}

.advanced-filter-row {
  display: flex;
  align-items: end;
  gap: 8px;
  flex-wrap: wrap;
  width: 100%;
}

.advanced-filter-row label {
  display: grid;
  gap: 4px;
  color: #8f9bb3;
  font-size: 12px;
}

.advanced-filter-row input,
.advanced-filter-row select {
  height: 32px;
  min-width: 88px;
  border: 1px solid #2d3447;
  border-radius: 6px;
  background: #171b25;
  color: #f4f7fb;
  padding: 0 8px;
}

.icon-text-btn {
  height: 32px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid #2d3447;
  border-radius: 6px;
  background: #1d2431;
  color: #dce5f5;
  padding: 0 10px;
  cursor: pointer;
}
```

- [ ] **Step 7: Run frontend verification**

Run:

```powershell
npm test -- src/components/SearchToolbar.test.tsx
npm run build
```

Expected: focused tests pass and production build succeeds.

- [ ] **Step 8: Commit**

```powershell
git add src/components/SearchToolbar.tsx src/components/SearchToolbar.test.tsx src/styles.css
git commit -m "feat: add search filter and sort controls"
```

---

### Task 58: Wire Filters And Sorting Into App Search

**Files:**
- Modify: `I:\GameResManger\src\App.tsx`

- [ ] **Step 1: Pass props to SearchToolbar**

Replace the current `SearchToolbar` call with:

```tsx
<SearchToolbar
  query={query}
  scope={scope}
  filters={filters}
  sort={sort}
  onQueryChange={(q) => { setQuery(q); setSelectedIds([]); }}
  onScopeChange={(s) => { setScope(s); setSelectedIds([]); }}
  onFiltersChange={handleFiltersChange}
  onSortChange={handleSortChange}
/>
```

- [ ] **Step 2: Update empty-state filter detection**

Replace:

```ts
const showSearchEmpty = query.trim() !== "" || activeFilter !== "all" || selectedFolderId != null || selectedCollectionId != null;
```

with:

```ts
const hasAdvancedFilters = Object.values(filters).some((value) => value != null);
const showSearchEmpty =
  query.trim() !== "" ||
  activeFilter !== "all" ||
  selectedFolderId != null ||
  selectedCollectionId != null ||
  hasAdvancedFilters;
```

If `showSearchEmpty` is currently unused, either use it to select the no-results empty state or remove the variable in a separate focused cleanup commit after verifying no behavior changes are needed.

- [ ] **Step 3: Ensure search request has all fields**

Confirm `req` includes:

```ts
min_file_size: filters.min_file_size,
max_file_size: filters.max_file_size,
min_width: filters.min_width,
max_width: filters.max_width,
min_height: filters.min_height,
max_height: filters.max_height,
modified_after: filters.modified_after,
modified_before: filters.modified_before,
sort_by: sort.sort_by,
sort_direction: sort.sort_direction,
```

- [ ] **Step 4: Run full frontend verification**

Run:

```powershell
npm test
npm run build
```

Expected: all frontend tests pass and production build succeeds.

- [ ] **Step 5: Commit**

```powershell
git add src/App.tsx src/types/asset.ts
git commit -m "feat: wire advanced asset search controls"
```

---

### Task 59: Add Smoke QA And Final Verification

**Files:**
- Modify: `I:\GameResManger\tests\smoke\README.md`

- [ ] **Step 1: Add manual smoke section**

Append this section to `tests/smoke/README.md`:

```markdown
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
```

- [ ] **Step 2: Run final automated verification**

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
- Backend compiles with no errors.

- [ ] **Step 3: Review working tree scope**

Run:

```powershell
git status --short
```

Expected: changes are limited to the files in this plan plus existing unrelated dirty worktree entries. Do not stage unrelated release artifacts, unrelated deleted files, or unrelated untracked files.

- [ ] **Step 4: Commit smoke documentation**

```powershell
git add tests/smoke/README.md
git commit -m "docs: add advanced filter smoke checks"
```

---

## Acceptance Criteria

- Users can filter by minimum and maximum file size.
- Users can filter image-like assets by width and height ranges.
- Users can filter by modified date range.
- Users can sort results by name, file size, modified time, or asset type.
- Sort direction can be ascending or descending.
- Backend SQL uses bound parameters for all user-provided filter values.
- Backend sorting uses a whitelist for column names and direction.
- Existing type, folder, collection, favorite, missing, query, tag, note, and path search behavior continues working.
- UI remains compact and tool-like inside the existing workbench.
- No source asset file is deleted, moved, renamed, modified, or written to.

## Self-Review

- Spec coverage:
  - Section 4.2 size, dimension, modified-time filters are covered by Tasks 54-58.
  - Section 4.2 sort controls are covered by Tasks 55, 57, and 58.
  - Section 10 backend and frontend verification are covered by Tasks 55, 57, 58, and 59.
- Scope check:
  - This is one subsystem: search/filter/sort. It does not include list view, AI tagging, atlas tools, file deletion, file movement, or full preview runtimes.
- No-placeholders scan:
  - The plan contains concrete file paths, code snippets, verification commands, expected results, and commit messages.
- Type consistency:
  - Rust `AssetSearchRequest` and TypeScript `AssetSearchRequest` use matching snake_case field names.
  - Frontend `AssetSearchSort` uses the same `sort_by` values whitelisted by Rust: `file_name`, `file_size`, `modified_at`, `asset_type`.
