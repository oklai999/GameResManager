import { RotateCcw, Search } from "lucide-react";
import type { AssetSearchFilters, AssetSearchSort, SearchScope } from "../types/asset";

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

export function SearchToolbar({
  query,
  scope,
  filters,
  sort,
  onQueryChange,
  onScopeChange,
  onFiltersChange,
  onSortChange,
}: Props) {
  function toggle(key: keyof SearchScope) {
    onScopeChange({ ...scope, [key]: !scope[key] });
  }

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

  const scopeItems: { key: keyof SearchScope; label: string }[] = [
    { key: "fileName", label: "文件名" },
    { key: "tag", label: "标签" },
    { key: "note", label: "备注" },
    { key: "path", label: "路径" },
  ];

  return (
    <header className="search-toolbar">
      <div className="search-input-wrap">
        <Search className="search-input-icon" size={16} aria-hidden="true" />
        <input
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          placeholder="搜索资源..."
          aria-label="搜索资源"
        />
      </div>
      <div className="scope-segments" aria-label="搜索范围">
        {scopeItems.map(({ key, label }) => (
          <button
            key={key}
            type="button"
            className={scope[key] ? "scope-segment active" : "scope-segment"}
            aria-pressed={scope[key]}
            onClick={() => toggle(key)}
          >
            {label}
          </button>
        ))}
      </div>
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
          <span>到</span>
          <input
            type="number"
            min="0"
            aria-label="最大宽度"
            value={filters.max_width ?? ""}
            onChange={(event) => updateNumberFilter("max_width", event.target.value)}
            placeholder="最大"
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
          <span>到</span>
          <input
            type="number"
            min="0"
            aria-label="最大高度"
            value={filters.max_height ?? ""}
            onChange={(event) => updateNumberFilter("max_height", event.target.value)}
            placeholder="最大"
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
    </header>
  );
}
