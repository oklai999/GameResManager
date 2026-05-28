import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { LibrarySidebar } from "./LibrarySidebar";

describe("LibrarySidebar", () => {
  it("calls onAddFolder when form is submitted", async () => {
    const onAddFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[]}
        activeFilter="all"
        onFilterChange={vi.fn()}
        onAddFolder={onAddFolder}
        onScanFolder={vi.fn()}
        error={null}
      />
    );

    await userEvent.type(screen.getByPlaceholderText("名称"), "test");
    await userEvent.type(screen.getByPlaceholderText("绝对路径"), "C:/assets");
    await userEvent.click(screen.getByText("添加"));

    expect(onAddFolder).toHaveBeenCalledWith("test", "C:/assets");
  });

  it("calls onScanFolder when scan button is clicked", async () => {
    const onScanFolder = vi.fn();
    render(
      <LibrarySidebar
        folders={[{ id: 1, name: "Assets", path: "C:/assets", created_at: "", last_scanned_at: null, is_enabled: true }]}
        activeFilter="all"
        onFilterChange={vi.fn()}
        onAddFolder={vi.fn()}
        onScanFolder={onScanFolder}
        error={null}
      />
    );

    await userEvent.click(screen.getByText("扫描"));

    expect(onScanFolder).toHaveBeenCalledWith(1);
  });
});
