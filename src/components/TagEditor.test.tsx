import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { TagEditor } from "./TagEditor";
import { listTags } from "../api/tauri";

vi.mock("../api/tauri", () => ({
  listTags: vi.fn(),
}));

const mockedListTags = vi.mocked(listTags);

describe("TagEditor", () => {
  beforeEach(() => {
    mockedListTags.mockResolvedValue([]);
  });

  it("renders existing tags as chips", async () => {
    render(<TagEditor existingTags={["important", "draft"]} onApply={vi.fn()} />);
    await waitFor(() => {
      expect(screen.getByText("important")).toBeInTheDocument();
    });
    expect(screen.getByText("draft")).toBeInTheDocument();
  });

  it("calls onApply with input value when form submitted", async () => {
    const onApply = vi.fn();
    render(<TagEditor existingTags={[]} onApply={onApply} />);

    const input = screen.getByPlaceholderText("输入标签...");
    await userEvent.type(input, "new-tag");
    await userEvent.click(screen.getByText("添加"));

    expect(onApply).toHaveBeenCalledWith("new-tag");
  });

  it("does not call onApply when input is empty", async () => {
    const onApply = vi.fn();
    render(<TagEditor existingTags={[]} onApply={onApply} />);

    await userEvent.click(screen.getByText("添加"));

    expect(onApply).not.toHaveBeenCalled();
  });

  it("shows recent tags before older tag suggestions", async () => {
    mockedListTags.mockResolvedValue([
      { id: 1, name: "角色", color: "#5B8DEF" },
      { id: 2, name: "地形", color: "#5B8DEF" },
    ]);

    render(<TagEditor existingTags={[]} recentTags={["特效", "角色"]} onApply={vi.fn()} />);

    await screen.findByRole("button", { name: "特效" });
    const suggestions = screen.getAllByRole("button").map((button) => button.textContent);

    expect(suggestions).toContain("特效");
    expect(suggestions.indexOf("特效")).toBeLessThan(suggestions.indexOf("角色"));
  });

  it("filters tag suggestions by typed text", async () => {
    mockedListTags.mockResolvedValue([
      { id: 1, name: "地形", color: "#5B8DEF" },
      { id: 2, name: "角色", color: "#5B8DEF" },
      { id: 3, name: "特效", color: "#5B8DEF" },
    ]);

    render(<TagEditor existingTags={[]} recentTags={["UI"]} onApply={vi.fn()} />);

    await userEvent.type(screen.getByPlaceholderText("输入标签..."), "地");

    expect(screen.getByRole("button", { name: "地形" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "角色" })).not.toBeInTheDocument();
  });

  it("does not suggest tags already applied to the current selection", async () => {
    mockedListTags.mockResolvedValue([
      { id: 1, name: "地形", color: "#5B8DEF" },
      { id: 2, name: "角色", color: "#5B8DEF" },
    ]);

    render(<TagEditor existingTags={["地形"]} recentTags={["地形", "角色"]} onApply={vi.fn()} />);

    await screen.findByRole("button", { name: "角色" });

    expect(screen.queryByRole("button", { name: "地形" })).not.toBeInTheDocument();
  });

  it("applies a suggested tag and clears the input", async () => {
    const onApply = vi.fn();
    mockedListTags.mockResolvedValue([{ id: 1, name: "特效", color: "#5B8DEF" }]);
    render(<TagEditor existingTags={[]} recentTags={[]} onApply={onApply} />);

    await userEvent.type(screen.getByPlaceholderText("输入标签..."), "特");
    await userEvent.click(screen.getByRole("button", { name: "特效" }));

    expect(onApply).toHaveBeenCalledWith("特效");
    expect(screen.getByPlaceholderText("输入标签...")).toHaveValue("");
  });
});
