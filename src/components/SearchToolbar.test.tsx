import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { SearchToolbar } from "./SearchToolbar";
import type { SearchScope } from "../types/asset";

const scope: SearchScope = {
  fileName: true,
  tag: true,
  note: true,
  path: false,
};

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

describe("SearchToolbar", () => {
  it("uses segmented scope buttons that toggle the search scope", async () => {
    const onScopeChange = vi.fn();
    render(
      <SearchToolbar
        query=""
        scope={scope}
        filters={filters}
        sort={sort}
        onQueryChange={vi.fn()}
        onScopeChange={onScopeChange}
        onFiltersChange={vi.fn()}
        onSortChange={vi.fn()}
      />
    );

    const pathButton = screen.getByRole("button", { name: "路径" });
    expect(pathButton).toHaveAttribute("aria-pressed", "false");

    await userEvent.click(pathButton);

    expect(onScopeChange).toHaveBeenCalledWith({ ...scope, path: true });
  });

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

  it("updates max width filter", async () => {
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

    fireEvent.change(screen.getByLabelText("最大宽度"), { target: { value: "256" } });

    expect(onFiltersChange).toHaveBeenCalledWith({ ...filters, max_width: 256 });
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

  it("explains literal substring search behavior", () => {
    render(
      <SearchToolbar
        query=""
        scope={scope}
        filters={filters}
        sort={sort}
        onQueryChange={vi.fn()}
        onScopeChange={vi.fn()}
        onFiltersChange={vi.fn()}
        onSortChange={vi.fn()}
      />
    );

    expect(
      screen.getByText("支持中文片段；空格分隔的关键词需同时匹配")
    ).toBeInTheDocument();
  });
});
