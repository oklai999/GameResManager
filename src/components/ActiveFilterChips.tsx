import { X } from "lucide-react";
import type { AssetSearchFilters, SearchScope } from "../types/asset";

type Chip = {
  id: string;
  label: string;
  onRemove: () => void;
};

type Props = {
  filters: AssetSearchFilters;
  scope: SearchScope;
  onFiltersChange: (filters: AssetSearchFilters) => void;
  onScopeChange: (scope: SearchScope) => void;
};

export function ActiveFilterChips({
  filters,
  scope,
  onFiltersChange,
  onScopeChange,
}: Props) {
  const chips: Chip[] = [];

  if (!scope.fileName) {
    chips.push({
      id: "scope-filename",
      label: "不搜文件名",
      onRemove: () => onScopeChange({ ...scope, fileName: true }),
    });
  }
  if (!scope.tag) {
    chips.push({
      id: "scope-tag",
      label: "不搜标签",
      onRemove: () => onScopeChange({ ...scope, tag: true }),
    });
  }
  if (!scope.note) {
    chips.push({
      id: "scope-note",
      label: "不搜备注",
      onRemove: () => onScopeChange({ ...scope, note: true }),
    });
  }
  if (!scope.path) {
    chips.push({
      id: "scope-path",
      label: "不搜路径",
      onRemove: () => onScopeChange({ ...scope, path: true }),
    });
  }

  if (filters.min_file_size != null && filters.min_file_size > 0) {
    const kb = Math.round(filters.min_file_size / 1024);
    chips.push({
      id: "min_file_size",
      label: `大小 ≥ ${kb} KB`,
      onRemove: () => onFiltersChange({ ...filters, min_file_size: null }),
    });
  }

  if (filters.max_file_size != null) {
    const kb = Math.round(filters.max_file_size / 1024);
    chips.push({
      id: "max_file_size",
      label: `大小 ≤ ${kb} KB`,
      onRemove: () => onFiltersChange({ ...filters, max_file_size: null }),
    });
  }

  if (filters.min_width != null && filters.min_width > 0) {
    chips.push({
      id: "min_width",
      label: `宽 ≥ ${filters.min_width}`,
      onRemove: () => onFiltersChange({ ...filters, min_width: null }),
    });
  }

  if (filters.max_width != null) {
    chips.push({
      id: "max_width",
      label: `宽 ≤ ${filters.max_width}`,
      onRemove: () => onFiltersChange({ ...filters, max_width: null }),
    });
  }

  if (filters.min_height != null && filters.min_height > 0) {
    chips.push({
      id: "min_height",
      label: `高 ≥ ${filters.min_height}`,
      onRemove: () => onFiltersChange({ ...filters, min_height: null }),
    });
  }

  if (filters.max_height != null) {
    chips.push({
      id: "max_height",
      label: `高 ≤ ${filters.max_height}`,
      onRemove: () => onFiltersChange({ ...filters, max_height: null }),
    });
  }

  if (filters.modified_after != null) {
    chips.push({
      id: "modified_after",
      label: `修改从 ${filters.modified_after.slice(0, 10)}`,
      onRemove: () => onFiltersChange({ ...filters, modified_after: null }),
    });
  }

  if (filters.modified_before != null) {
    chips.push({
      id: "modified_before",
      label: `修改到 ${filters.modified_before.slice(0, 10)}`,
      onRemove: () => onFiltersChange({ ...filters, modified_before: null }),
    });
  }

  if (chips.length === 0) return null;

  return (
    <div className="filter-chips" role="list" aria-label="活跃筛选">
      {chips.map((chip) => (
        <span
          key={chip.id}
          className="filter-chip"
          role="listitem"
          data-testid="filter-chip"
        >
          <span className="filter-chip-label">{chip.label}</span>
          <button
            type="button"
            className="filter-chip-remove"
            aria-label={`移除筛选: ${chip.label}`}
            onClick={chip.onRemove}
          >
            <X size={12} aria-hidden="true" />
          </button>
        </span>
      ))}
    </div>
  );
}
