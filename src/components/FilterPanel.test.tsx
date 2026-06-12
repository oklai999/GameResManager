import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { FilterPanel } from "./FilterPanel";
import type { AssetSearchFilters, SearchScope } from "../types/asset";

const defaultScope: SearchScope = {
  fileName: true,
  tag: true,
  note: true,
  path: false,
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

describe("FilterPanel", () => {
  it("renders nothing when open is false", () => {
    const { container } = render(
      <FilterPanel
        open={false}
        scope={defaultScope}
        filters={defaultFilters}
        onScopeChange={vi.fn()}
        onFiltersChange={vi.fn()}
      />
    );
    expect(container.textContent).toBe("");
  });

  it("renders panel with title when open is true", () => {
    render(
      <FilterPanel
        open={true}
        scope={defaultScope}
        filters={defaultFilters}
        onScopeChange={vi.fn()}
        onFiltersChange={vi.fn()}
      />
    );
    expect(screen.getByText("高级筛选")).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "高级筛选" })).toBeInTheDocument();
  });

  it("toggles scope segments", async () => {
    const onScopeChange = vi.fn();
    render(
      <FilterPanel
        open={true}
        scope={defaultScope}
        filters={defaultFilters}
        onScopeChange={onScopeChange}
        onFiltersChange={vi.fn()}
      />
    );

    const pathButton = screen.getByRole("button", { name: "路径" });
    expect(pathButton).toHaveAttribute("aria-pressed", "false");

    await userEvent.click(pathButton);

    expect(onScopeChange).toHaveBeenCalledWith({ ...defaultScope, path: true });
  });

  it("updates file size min filter", () => {
    const onFiltersChange = vi.fn();
    render(
      <FilterPanel
        open={true}
        scope={defaultScope}
        filters={defaultFilters}
        onScopeChange={vi.fn()}
        onFiltersChange={onFiltersChange}
      />
    );

    fireEvent.change(screen.getByLabelText("最小大小 KB"), {
      target: { value: "128" },
    });

    expect(onFiltersChange).toHaveBeenCalledWith({
      ...defaultFilters,
      min_file_size: 128 * 1024,
    });
  });

  it("updates max width filter", () => {
    const onFiltersChange = vi.fn();
    render(
      <FilterPanel
        open={true}
        scope={defaultScope}
        filters={defaultFilters}
        onScopeChange={vi.fn()}
        onFiltersChange={onFiltersChange}
      />
    );

    fireEvent.change(screen.getByLabelText("最大宽度"), {
      target: { value: "256" },
    });

    expect(onFiltersChange).toHaveBeenCalledWith({
      ...defaultFilters,
      max_width: 256,
    });
  });

  it("updates date filters", () => {
    const onFiltersChange = vi.fn();
    render(
      <FilterPanel
        open={true}
        scope={defaultScope}
        filters={defaultFilters}
        onScopeChange={vi.fn()}
        onFiltersChange={onFiltersChange}
      />
    );

    fireEvent.change(screen.getByLabelText("修改日期从"), {
      target: { value: "2025-01-15" },
    });

    expect(onFiltersChange).toHaveBeenCalledWith({
      ...defaultFilters,
      modified_after: "2025-01-15T00:00:00Z",
    });
  });

  it("resets all advanced filters when reset button is clicked", async () => {
    const onFiltersChange = vi.fn();
    const activeFilters: AssetSearchFilters = {
      ...defaultFilters,
      min_width: 64,
      max_width: 256,
    };
    render(
      <FilterPanel
        open={true}
        scope={defaultScope}
        filters={activeFilters}
        onScopeChange={vi.fn()}
        onFiltersChange={onFiltersChange}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "重置高级筛选" }));

    expect(onFiltersChange).toHaveBeenCalledWith(defaultFilters);
  });
});
