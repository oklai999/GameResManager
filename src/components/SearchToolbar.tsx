import { ListFilter, LayoutGrid, LayoutList, Search } from "lucide-react";
import type { AssetSearchSort } from "../types/asset";

export type GridDensity = "compact" | "comfortable";

type Props = {
  query: string;
  sort: AssetSearchSort;
  totalCount: number;
  filterOpen: boolean;
  density: GridDensity;
  onQueryChange: (query: string) => void;
  onSortChange: (sort: AssetSearchSort) => void;
  onToggleFilters: () => void;
  onDensityChange: (density: "compact" | "comfortable") => void;
};

export function SearchToolbar({
  query,
  sort,
  totalCount,
  filterOpen,
  density,
  onQueryChange,
  onSortChange,
  onToggleFilters,
  onDensityChange,
}: Props) {
  return (
    <header className="search-toolbar">
      <div className="search-query-group">
        <div className="search-input-wrap">
          <Search className="search-input-icon" size={16} aria-hidden="true" />
          <input
            value={query}
            onChange={(event) => onQueryChange(event.target.value)}
            placeholder="搜索资源..."
            aria-label="搜索资源"
          />
        </div>
        <span className="search-hint">
          支持中文片段；空格分隔的关键词需同时匹配
        </span>
      </div>
      <span className="result-count">{totalCount.toLocaleString()} 项</span>
      <button
        type="button"
        className="icon-text-btn"
        aria-expanded={filterOpen}
        aria-controls="filter-panel"
        aria-label="筛选"
        onClick={onToggleFilters}
      >
        <ListFilter size={16} aria-hidden="true" />
      </button>
      <select
        className="toolbar-select"
        aria-label="排序字段"
        value={sort.sort_by}
        onChange={(event) =>
          onSortChange({
            ...sort,
            sort_by: event.target.value as AssetSearchSort["sort_by"],
          })
        }
      >
        <option value="file_name">名称</option>
        <option value="file_size">大小</option>
        <option value="modified_at">修改时间</option>
        <option value="asset_type">类型</option>
      </select>
      <select
        className="toolbar-select"
        aria-label="排序方向"
        value={sort.sort_direction}
        onChange={(event) =>
          onSortChange({
            ...sort,
            sort_direction: event.target.value as AssetSearchSort["sort_direction"],
          })
        }
      >
        <option value="asc">升序</option>
        <option value="desc">降序</option>
      </select>
      <button
        type="button"
        className="icon-text-btn"
        aria-label={density === "compact" ? "舒适网格" : "紧凑网格"}
        onClick={() =>
          onDensityChange(density === "compact" ? "comfortable" : "compact")
        }
      >
        {density === "compact" ? (
          <LayoutList size={16} aria-hidden="true" />
        ) : (
          <LayoutGrid size={16} aria-hidden="true" />
        )}
        {density === "compact" ? "舒适网格" : "紧凑网格"}
      </button>
    </header>
  );
}
