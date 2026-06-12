import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ActiveFilterChips } from "./ActiveFilterChips";
import type { AssetSearchFilters, SearchScope } from "../types/asset";

const defaultScope: SearchScope = {
  fileName: true,
  tag: true,
  note: true,
  path: true,
};

const defaultFilters: AssetSearchFilters = {
  min_file_size: null,
  max_file_size: null,
  min_width: null,
  max_width: null,
  min_height: null,
  max_height: null,
  modified_after: null,
  modified_before: null,
};

describe("ActiveFilterChips", () => {
  it("renders no chips when all filters are default and scope is all-enabled", () => {
    const { container } = render(
      <ActiveFilterChips
        filters={defaultFilters}
        scope={defaultScope}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );
    expect(container.textContent).toBe("");
  });

  it("shows chips for disabled scope items", () => {
    render(
      <ActiveFilterChips
        filters={defaultFilters}
        scope={{ fileName: false, tag: false, note: false, path: false }}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );
    expect(screen.getByText("不搜文件名")).toBeInTheDocument();
    expect(screen.getByText("不搜标签")).toBeInTheDocument();
    expect(screen.getByText("不搜备注")).toBeInTheDocument();
    expect(screen.getByText("不搜路径")).toBeInTheDocument();
  });

  it("shows chip for disabled path scope", () => {
    render(
      <ActiveFilterChips
        filters={defaultFilters}
        scope={{ ...defaultScope, path: false }}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );
    expect(screen.getByText("不搜路径")).toBeInTheDocument();
  });

  it("clicking close on disabled-path chip calls onScopeChange to restore path", async () => {
    const onScopeChange = vi.fn();
    render(
      <ActiveFilterChips
        filters={defaultFilters}
        scope={{ ...defaultScope, path: false }}
        onFiltersChange={vi.fn()}
        onScopeChange={onScopeChange}
      />
    );

    await userEvent.click(screen.getByLabelText("移除筛选: 不搜路径"));

    expect(onScopeChange).toHaveBeenCalledWith({ ...defaultScope, path: true });
  });

  it("shows chip for min_file_size", () => {
    render(
      <ActiveFilterChips
        filters={{ ...defaultFilters, min_file_size: 102400 }}
        scope={defaultScope}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );
    expect(screen.getByText("大小 ≥ 100 KB")).toBeInTheDocument();
  });

  it("shows chip for max_file_size", () => {
    render(
      <ActiveFilterChips
        filters={{ ...defaultFilters, max_file_size: 51200 }}
        scope={defaultScope}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );
    expect(screen.getByText("大小 ≤ 50 KB")).toBeInTheDocument();
  });

  it("shows chip for min_width", () => {
    render(
      <ActiveFilterChips
        filters={{ ...defaultFilters, min_width: 128 }}
        scope={defaultScope}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );
    expect(screen.getByText("宽 ≥ 128")).toBeInTheDocument();
  });

  it("shows chip for max_width", () => {
    render(
      <ActiveFilterChips
        filters={{ ...defaultFilters, max_width: 640 }}
        scope={defaultScope}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );
    expect(screen.getByText("宽 ≤ 640")).toBeInTheDocument();
  });

  it("shows chip for min_height", () => {
    render(
      <ActiveFilterChips
        filters={{ ...defaultFilters, min_height: 72 }}
        scope={defaultScope}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );
    expect(screen.getByText("高 ≥ 72")).toBeInTheDocument();
  });

  it("shows chip for max_height", () => {
    render(
      <ActiveFilterChips
        filters={{ ...defaultFilters, max_height: 480 }}
        scope={defaultScope}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );
    expect(screen.getByText("高 ≤ 480")).toBeInTheDocument();
  });

  it("shows chip for modified_after", () => {
    render(
      <ActiveFilterChips
        filters={{ ...defaultFilters, modified_after: "2025-01-15T00:00:00Z" }}
        scope={defaultScope}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );
    expect(screen.getByText("修改从 2025-01-15")).toBeInTheDocument();
  });

  it("shows chip for modified_before", () => {
    render(
      <ActiveFilterChips
        filters={{ ...defaultFilters, modified_before: "2025-06-01T00:00:00Z" }}
        scope={defaultScope}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );
    expect(screen.getByText("修改到 2025-06-01")).toBeInTheDocument();
  });

  it("removes a single filter chip without affecting others", async () => {
    const onFiltersChange = vi.fn();
    render(
      <ActiveFilterChips
        filters={{
          ...defaultFilters,
          min_file_size: 102400,
          min_width: 128,
        }}
        scope={defaultScope}
        onFiltersChange={onFiltersChange}
        onScopeChange={vi.fn()}
      />
    );

    expect(screen.getByText("大小 ≥ 100 KB")).toBeInTheDocument();
    expect(screen.getByText("宽 ≥ 128")).toBeInTheDocument();

    await userEvent.click(screen.getByLabelText("移除筛选: 大小 ≥ 100 KB"));

    expect(onFiltersChange).toHaveBeenCalledWith({
      ...defaultFilters,
      min_file_size: null,
      min_width: 128,
    });
  });

  it("renders all filter types together without conflict", () => {
    const allFilters: AssetSearchFilters = {
      min_file_size: 102400,
      max_file_size: 512000,
      min_width: 128,
      max_width: 640,
      min_height: 72,
      max_height: 480,
      modified_after: "2025-01-01T00:00:00Z",
      modified_before: "2025-12-31T00:00:00Z",
    };

    render(
      <ActiveFilterChips
        filters={allFilters}
        scope={{ ...defaultScope, path: false }}
        onFiltersChange={vi.fn()}
        onScopeChange={vi.fn()}
      />
    );

    expect(screen.getByText("不搜路径")).toBeInTheDocument();
    expect(screen.getByText("大小 ≥ 100 KB")).toBeInTheDocument();
    expect(screen.getByText("大小 ≤ 500 KB")).toBeInTheDocument();
    expect(screen.getByText("宽 ≥ 128")).toBeInTheDocument();
    expect(screen.getByText("宽 ≤ 640")).toBeInTheDocument();
    expect(screen.getByText("高 ≥ 72")).toBeInTheDocument();
    expect(screen.getByText("高 ≤ 480")).toBeInTheDocument();
    expect(screen.getByText("修改从 2025-01-01")).toBeInTheDocument();
    expect(screen.getByText("修改到 2025-12-31")).toBeInTheDocument();
  });
});
