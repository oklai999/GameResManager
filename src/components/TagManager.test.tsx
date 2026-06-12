import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { TagManager } from "./TagManager";
import type { Tag } from "../types/asset";

const tags: Tag[] = [
  { id: 1, name: "角色", color: "#5B8DEF", asset_count: 12 },
  { id: 2, name: "特效", color: "#EF8354", asset_count: 4 },
];

describe("TagManager", () => {
  it("shows name color and asset count", () => {
    render(<TagManager tags={tags} onClose={vi.fn()} onUpdate={vi.fn()} onDelete={vi.fn()} />);
    expect(screen.getByText("角色")).toBeInTheDocument();
    expect(screen.getByText("12")).toBeInTheDocument();
    expect(screen.getByLabelText("标签颜色")).toHaveValue("#5B8DEF");
  });

  it("submits edited name and color", async () => {
    const onUpdate = vi.fn();
    render(<TagManager tags={tags} onClose={vi.fn()} onUpdate={onUpdate} onDelete={vi.fn()} />);
    await userEvent.clear(screen.getByLabelText("标签名称"));
    await userEvent.type(screen.getByLabelText("标签名称"), "主角");
    await userEvent.clear(screen.getByLabelText("标签颜色"));
    await userEvent.type(screen.getByLabelText("标签颜色"), "#abcdef");
    await userEvent.click(screen.getByRole("button", { name: "保存标签" }));
    expect(onUpdate).toHaveBeenCalledWith(1, "主角", "#abcdef");
  });

  it("requires explicit confirmation before delete", async () => {
    const onDelete = vi.fn();
    render(<TagManager tags={tags} onClose={vi.fn()} onUpdate={vi.fn()} onDelete={onDelete} />);
    await userEvent.click(screen.getByRole("button", { name: "删除标签" }));
    expect(screen.getByText(/不会删除或修改原始素材文件/)).toBeInTheDocument();
    expect(onDelete).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "确认删除标签" }));
    expect(onDelete).toHaveBeenCalledWith(1);
  });

  it("falls back when the selected tag is merged or deleted", () => {
    const { rerender } = render(
      <TagManager tags={tags} onClose={vi.fn()} onUpdate={vi.fn()} onDelete={vi.fn()} />
    );
    rerender(
      <TagManager tags={[tags[1]]} onClose={vi.fn()} onUpdate={vi.fn()} onDelete={vi.fn()} />
    );
    expect(screen.getByLabelText("标签名称")).toHaveValue("特效");
  });

  it("shows empty state when there are no tags", () => {
    render(<TagManager tags={[]} onClose={vi.fn()} onUpdate={vi.fn()} onDelete={vi.fn()} />);
    expect(screen.queryByLabelText("标签名称")).not.toBeInTheDocument();
    expect(screen.getByText(/还没有标签/)).toBeInTheDocument();
  });
});
