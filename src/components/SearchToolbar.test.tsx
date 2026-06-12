import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { SearchToolbar } from "./SearchToolbar";
import type { AssetSearchSort } from "../types/asset";

const sort: AssetSearchSort = {
  sort_by: "file_name",
  sort_direction: "asc",
};

describe("SearchToolbar", () => {
  it("shows the result count", () => {
    render(
      <SearchToolbar
        query=""
        sort={sort}
        totalCount={12458}
        filterOpen={false}
        density="compact"
        onQueryChange={vi.fn()}
        onSortChange={vi.fn()}
        onToggleFilters={vi.fn()}
        onDensityChange={vi.fn()}
      />
    );

    expect(screen.getByText("12,458 项")).toBeInTheDocument();
  });

  it("filter button shows aria-expanded state from prop", () => {
    const { rerender } = render(
      <SearchToolbar
        query=""
        sort={sort}
        totalCount={0}
        filterOpen={false}
        density="compact"
        onQueryChange={vi.fn()}
        onSortChange={vi.fn()}
        onToggleFilters={vi.fn()}
        onDensityChange={vi.fn()}
      />
    );

    const filterBtn = screen.getByRole("button", { name: "筛选" });
    expect(filterBtn).toHaveAttribute("aria-expanded", "false");

    rerender(
      <SearchToolbar
        query=""
        sort={sort}
        totalCount={0}
        filterOpen={true}
        density="compact"
        onQueryChange={vi.fn()}
        onSortChange={vi.fn()}
        onToggleFilters={vi.fn()}
        onDensityChange={vi.fn()}
      />
    );

    expect(filterBtn).toHaveAttribute("aria-expanded", "true");
  });

  it("calls onToggleFilters when filter button is clicked", async () => {
    const onToggleFilters = vi.fn();
    render(
      <SearchToolbar
        query=""
        sort={sort}
        totalCount={0}
        filterOpen={false}
        density="compact"
        onQueryChange={vi.fn()}
        onSortChange={vi.fn()}
        onToggleFilters={onToggleFilters}
        onDensityChange={vi.fn()}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "筛选" }));

    expect(onToggleFilters).toHaveBeenCalledOnce();
  });

  it("toggles density from compact to comfortable", async () => {
    const onDensityChange = vi.fn();
    render(
      <SearchToolbar
        query=""
        sort={sort}
        totalCount={0}
        filterOpen={false}
        density="compact"
        onQueryChange={vi.fn()}
        onSortChange={vi.fn()}
        onToggleFilters={vi.fn()}
        onDensityChange={onDensityChange}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "舒适网格" }));

    expect(onDensityChange).toHaveBeenCalledWith("comfortable");
  });

  it("toggles density from comfortable to compact", async () => {
    const onDensityChange = vi.fn();
    render(
      <SearchToolbar
        query=""
        sort={sort}
        totalCount={0}
        filterOpen={false}
        density="comfortable"
        onQueryChange={vi.fn()}
        onSortChange={vi.fn()}
        onToggleFilters={vi.fn()}
        onDensityChange={onDensityChange}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "紧凑网格" }));

    expect(onDensityChange).toHaveBeenCalledWith("compact");
  });

  it("updates sort controls", async () => {
    const onSortChange = vi.fn();
    render(
      <SearchToolbar
        query=""
        sort={sort}
        totalCount={0}
        filterOpen={false}
        density="compact"
        onQueryChange={vi.fn()}
        onSortChange={onSortChange}
        onToggleFilters={vi.fn()}
        onDensityChange={vi.fn()}
      />
    );

    await userEvent.selectOptions(screen.getByLabelText("排序字段"), "file_size");
    await userEvent.selectOptions(screen.getByLabelText("排序方向"), "desc");

    expect(onSortChange).toHaveBeenCalledWith({ ...sort, sort_by: "file_size" });
    expect(onSortChange).toHaveBeenCalledWith({ ...sort, sort_direction: "desc" });
  });

  it("explains literal substring search behavior", () => {
    render(
      <SearchToolbar
        query=""
        sort={sort}
        totalCount={0}
        filterOpen={false}
        density="compact"
        onQueryChange={vi.fn()}
        onSortChange={vi.fn()}
        onToggleFilters={vi.fn()}
        onDensityChange={vi.fn()}
      />
    );

    expect(
      screen.getByText("支持中文片段；空格分隔的关键词需同时匹配"),
    ).toBeInTheDocument();
  });
});
