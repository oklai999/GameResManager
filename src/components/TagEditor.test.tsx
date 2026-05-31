import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { TagEditor } from "./TagEditor";

vi.mock("../api/tauri", () => ({
  listTags: vi.fn().mockResolvedValue([]),
}));

describe("TagEditor", () => {
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
});
