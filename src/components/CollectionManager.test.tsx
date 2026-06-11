import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { CollectionManager } from "./CollectionManager";
import type { Collection } from "../types/asset";

const collection: Collection = {
  id: 1,
  name: "角色",
  description: "常用角色素材",
  asset_count: 12,
};

describe("CollectionManager", () => {
  it("edits collection name and description", async () => {
    const onUpdate = vi.fn();
    render(
      <CollectionManager
        collections={[collection]}
        onClose={vi.fn()}
        onUpdate={onUpdate}
        onDelete={vi.fn()}
      />
    );

    await userEvent.clear(screen.getByLabelText("集合名称"));
    await userEvent.type(screen.getByLabelText("集合名称"), "主角");
    await userEvent.clear(screen.getByLabelText("集合描述"));
    await userEvent.type(screen.getByLabelText("集合描述"), "主角动画与立绘");
    await userEvent.click(screen.getByRole("button", { name: "保存集合" }));

    expect(onUpdate).toHaveBeenCalledWith(1, "主角", "主角动画与立绘");
  });

  it("requires explicit confirmation before deleting", async () => {
    const onDelete = vi.fn();
    render(
      <CollectionManager
        collections={[collection]}
        onClose={vi.fn()}
        onUpdate={vi.fn()}
        onDelete={onDelete}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "删除集合" }));
    expect(screen.getByText(/不会删除任何原始素材文件/)).toBeInTheDocument();
    expect(onDelete).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole("button", { name: "确认删除集合" }));
    expect(onDelete).toHaveBeenCalledWith(1);
  });

  it("auto-selects next collection after deleting current selection", async () => {
    const onDelete = vi.fn();
    const { rerender } = render(
      <CollectionManager
        collections={[
          { id: 1, name: "A", description: "desc-a", asset_count: 1 },
          { id: 2, name: "B", description: "desc-b", asset_count: 2 },
        ]}
        onClose={vi.fn()}
        onUpdate={vi.fn()}
        onDelete={onDelete}
      />
    );

    expect(screen.getByDisplayValue("desc-a")).toBeInTheDocument();

    rerender(
      <CollectionManager
        collections={[{ id: 2, name: "B", description: "desc-b", asset_count: 2 }]}
        onClose={vi.fn()}
        onUpdate={vi.fn()}
        onDelete={onDelete}
      />
    );

    expect(screen.queryByDisplayValue("desc-a")).not.toBeInTheDocument();
    expect(screen.getByDisplayValue("desc-b")).toBeInTheDocument();
  });

  it("shows empty state after deleting the last collection", async () => {
    const onDelete = vi.fn();
    const { rerender } = render(
      <CollectionManager
        collections={[collection]}
        onClose={vi.fn()}
        onUpdate={vi.fn()}
        onDelete={onDelete}
      />
    );

    expect(screen.getByLabelText("集合名称")).toBeInTheDocument();

    rerender(
      <CollectionManager
        collections={[]}
        onClose={vi.fn()}
        onUpdate={vi.fn()}
        onDelete={onDelete}
      />
    );

    expect(screen.queryByLabelText("集合名称")).not.toBeInTheDocument();
    expect(screen.getByText(/还没有创建集合/)).toBeInTheDocument();
  });

  it("auto-selects first collection when collections go from empty to non-empty", async () => {
    const { rerender } = render(
      <CollectionManager
        collections={[]}
        onClose={vi.fn()}
        onUpdate={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    expect(screen.getByText(/还没有创建集合/)).toBeInTheDocument();

    rerender(
      <CollectionManager
        collections={[collection]}
        onClose={vi.fn()}
        onUpdate={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    expect(screen.getByDisplayValue("常用角色素材")).toBeInTheDocument();
  });
});
