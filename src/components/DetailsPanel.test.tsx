import { render, screen, waitFor } from "@testing-library/react";
import userEvent, { UserEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { DetailsPanel } from "./DetailsPanel";
import type { Asset, Collection } from "../types/asset";
import { getAssetTags, listCommonTags } from "../api/tauri";

const mockedGetAssetTags = vi.mocked(getAssetTags);
const mockedListCommonTags = vi.mocked(listCommonTags);

vi.mock("../api/tauri", () => ({
  getAssetTags: vi.fn().mockResolvedValue([]),
  listCommonTags: vi.fn().mockResolvedValue([]),
  listTags: vi.fn().mockResolvedValue([]),
  updateAssetNote: vi.fn().mockResolvedValue({}),
  assetPathVariants: vi.fn().mockResolvedValue({
    absolute_path: "",
    forward_slash_path: "",
    folder_path: "",
    file_name: "",
    godot_res_path: null,
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => path,
}));

function makeAsset(id: number): Asset {
  return {
    id,
    library_folder_id: 1,
    absolute_path: `C:/assets/${id}.png`,
    file_name: `${id}.png`,
    extension: "png",
    asset_type: "image",
    file_size: 100,
    modified_at: "2026-05-27T00:00:00Z",
    width: 128,
    height: 128,
    thumbnail_path: null,
    thumbnail_status: "none" as const,
    thumbnail_error: null,
    note: "",
    is_favorite: false,
    is_missing: false,
    created_at: "2026-05-27T00:00:00Z",
    updated_at: "2026-05-27T00:00:00Z",
  };
}

function makeCollection(id: number, name: string): Collection {
  return { id, name, description: "", asset_count: 0 };
}

describe("DetailsPanel", () => {
  it("shows batch mode for multiple selected assets", async () => {
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1), makeAsset(2)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    expect(await screen.findByText("已选择 2 个资源")).toBeInTheDocument();
  });

  it("shows single asset details", async () => {
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    expect(await screen.findByText("1.png")).toBeInTheDocument();
  });

  it("shows thumbnail error for failed thumbnails", async () => {
    const asset = makeAsset(1);
    asset.thumbnail_status = "failed";
    asset.thumbnail_error = "Invalid PNG signature";

    render(
      <DetailsPanel
        selectedAssets={[asset]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    expect(await screen.findByText("缩略图错误：Invalid PNG signature")).toBeInTheDocument();
  });

  it("shows thumbnail cache path for ready thumbnails", async () => {
    const asset = makeAsset(1);
    asset.thumbnail_status = "ready";
    asset.thumbnail_path = "C:/cache/icon.webp";

    render(
      <DetailsPanel
        selectedAssets={[asset]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    expect(await screen.findByText("缩略图缓存：C:/cache/icon.webp")).toBeInTheDocument();
  });

  it("calls onApplyTag when tag is submitted in single mode", async () => {
    const onApplyTag = vi.fn();
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={onApplyTag}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    const input = await screen.findByPlaceholderText("输入标签...");
    await userEvent.type(input, "hero");
    await userEvent.click(screen.getByText("添加"));

    await waitFor(() => {
      expect(onApplyTag).toHaveBeenCalledWith("hero", [1]);
    });
  });

  it("shows collection buttons and calls onAddToCollection for single asset", async () => {
    const onAddToCollection = vi.fn();
    const collections = [makeCollection(1, "Heroes"), makeCollection(2, "Icons")];

    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={collections}
        onAddToCollection={onAddToCollection}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    const heroesBtn = await screen.findByText("Heroes");
    expect(screen.getByText("Icons")).toBeInTheDocument();
    await userEvent.click(heroesBtn);

    expect(onAddToCollection).toHaveBeenCalledWith(1, [1]);
  });

  it("calls onAddToCollection with all selected ids in batch mode", async () => {
    const onAddToCollection = vi.fn();
    const collections = [makeCollection(1, "Heroes")];

    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1), makeAsset(2)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={collections}
        onAddToCollection={onAddToCollection}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    await userEvent.click(await screen.findByText("Heroes"));

    expect(onAddToCollection).toHaveBeenCalledWith(1, [1, 2]);
  });

  it("calls onCopyPath when copy button is clicked", async () => {
    const onCopyPath = vi.fn();
    const asset = makeAsset(1);

    render(
      <DetailsPanel
        selectedAssets={[asset]}
        collections={[]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={onCopyPath}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "复制路径" }));
    expect(onCopyPath).toHaveBeenCalledWith(asset);
  });

  it("shows path variant copy buttons for single asset", async () => {
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        collections={[]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        onCopyText={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    expect(await screen.findByRole("button", { name: "复制绝对路径" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "复制正斜杠路径" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "复制文件夹路径" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "复制文件名" })).toBeInTheDocument();
  });

  it("calls onCopyText with file name when copy file name button is clicked", async () => {
    const onCopyText = vi.fn();
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        collections={[]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        onCopyText={onCopyText}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    await userEvent.click(await screen.findByRole("button", { name: "复制文件名" }));
    expect(onCopyText).toHaveBeenCalledWith("1.png");
  });

  it("does not show path variant buttons in batch mode", async () => {
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1), makeAsset(2)]}
        collections={[]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        onCopyText={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    expect(await screen.findByText("已选择 2 个资源")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "复制绝对路径" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "复制文件名" })).not.toBeInTheDocument();
  });

  it("renders note textarea and calls onUpdateNote after saving", async () => {
    const { updateAssetNote } = await import("../api/tauri");
    const onUpdateNote = vi.fn();
    const asset = makeAsset(1);
    asset.note = "old note";

    const updatedAsset = { ...asset, note: "new note" };
    vi.mocked(updateAssetNote).mockResolvedValueOnce(updatedAsset);

    render(
      <DetailsPanel
        selectedAssets={[asset]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        onUpdateNote={onUpdateNote}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    const textarea = await screen.findByDisplayValue("old note");
    await userEvent.clear(textarea);
    await userEvent.type(textarea, "new note");
    await userEvent.click(screen.getByText("保存备注"));

    await waitFor(() => {
      expect(updateAssetNote).toHaveBeenCalledWith(1, "new note");
    });
    await waitFor(() => {
      expect(onUpdateNote).toHaveBeenCalledWith(updatedAsset);
    });
  });

  it("does not show res:// button without project root", async () => {
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        collections={[]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        onCopyText={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    expect(await screen.findByRole("button", { name: "复制绝对路径" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "复制 res:// 路径" })).not.toBeInTheDocument();
    expect(screen.queryByText("不在项目根目录下")).not.toBeInTheDocument();
  });

  it("shows res:// button when asset is inside project root", async () => {
    const { assetPathVariants } = await import("../api/tauri");
    const onCopyText = vi.fn();
    vi.mocked(assetPathVariants).mockResolvedValueOnce({
      absolute_path: "C:/project/assets/1.png",
      forward_slash_path: "C:/project/assets/1.png",
      folder_path: "C:/project/assets",
      file_name: "1.png",
      godot_res_path: "res://assets/1.png",
    });

    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        collections={[]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        onCopyText={onCopyText}
        projectRoot="C:/project"
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    const resBtn = await screen.findByRole("button", { name: "复制 res:// 路径" });
    expect(resBtn).toBeInTheDocument();
    await userEvent.click(resBtn);
    expect(onCopyText).toHaveBeenCalledWith("res://assets/1.png");
  });

  it("shows not-in-project message when asset is outside project root", async () => {
    const { assetPathVariants } = await import("../api/tauri");
    vi.mocked(assetPathVariants).mockResolvedValueOnce({
      absolute_path: "C:/other/assets/1.png",
      forward_slash_path: "C:/other/assets/1.png",
      folder_path: "C:/other/assets",
      file_name: "1.png",
      godot_res_path: null,
    });

    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        collections={[]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        onCopyText={vi.fn()}
        projectRoot="C:/project"
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    expect(await screen.findByText("不在项目根目录下")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "复制 res:// 路径" })).not.toBeInTheDocument();
  });

  it("existing absolute copy button still exists when project root is set", async () => {
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        collections={[]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        onCopyText={vi.fn()}
        projectRoot="C:/project"
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    expect(await screen.findByRole("button", { name: "复制绝对路径" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "复制正斜杠路径" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "复制文件夹路径" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "复制文件名" })).toBeInTheDocument();
  });

  it("clears old res:// button immediately when projectRoot changes", async () => {
    const { assetPathVariants } = await import("../api/tauri");
    vi.mocked(assetPathVariants).mockResolvedValueOnce({
      absolute_path: "C:/project/assets/1.png",
      forward_slash_path: "C:/project/assets/1.png",
      folder_path: "C:/project/assets",
      file_name: "1.png",
      godot_res_path: "res://assets/1.png",
    });
    vi.mocked(assetPathVariants).mockImplementation(() => new Promise(() => {}));

    const { rerender } = render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        collections={[]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        onCopyText={vi.fn()}
        projectRoot="C:/project"
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    await screen.findByRole("button", { name: "复制 res:// 路径" });

    rerender(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        collections={[]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        onCopyText={vi.fn()}
        projectRoot="C:/other"
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    await waitFor(() => {
      expect(screen.queryByRole("button", { name: "复制 res:// 路径" })).not.toBeInTheDocument();
    });
  });

  it("shows remove-from-collection button in single mode when activeCollectionId is set", async () => {
    const onRemoveFromCollection = vi.fn();
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        collections={[makeCollection(1, "Heroes")]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        activeCollectionId={1}
        onRemoveFromCollection={onRemoveFromCollection}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    const btn = await screen.findByRole("button", { name: "从当前集合移出" });
    await userEvent.click(btn);

    expect(onRemoveFromCollection).toHaveBeenCalledWith(1, [1]);
  });

  it("shows batch remove-from-collection button in batch mode when activeCollectionId is set", async () => {
    const onRemoveFromCollection = vi.fn();
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1), makeAsset(2)]}
        collections={[makeCollection(1, "Heroes")]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        activeCollectionId={1}
        onRemoveFromCollection={onRemoveFromCollection}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    const btn = await screen.findByRole("button", { name: "从当前集合移出 2 个资源" });
    await userEvent.click(btn);

    expect(onRemoveFromCollection).toHaveBeenCalledWith(1, [1, 2]);
  });

  it("does not show remove-from-collection button when activeCollectionId is null", async () => {
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        collections={[makeCollection(1, "Heroes")]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );

    expect(await screen.findByText("1.png")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "从当前集合移出" })).not.toBeInTheDocument();
  });

  it("removes a tag from one selected asset", async () => {
    mockedGetAssetTags.mockResolvedValue(["角色"]);
    const onRemoveTag = vi.fn();
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={onRemoveTag}
        tagRefreshVersion={0}
      />
    );
    await screen.findByRole("button", { name: "移除标签 角色" });
    await userEvent.click(screen.getByRole("button", { name: "移除标签 角色" }));
    expect(onRemoveTag).toHaveBeenCalledWith("角色", [1]);
  });

  it("removes a common tag from all selected assets", async () => {
    mockedListCommonTags.mockResolvedValue(["角色"]);
    const onRemoveTag = vi.fn();
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1), makeAsset(2)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={onRemoveTag}
        tagRefreshVersion={0}
      />
    );
    await screen.findByRole("button", { name: "移除标签 角色" });
    await userEvent.click(screen.getByRole("button", { name: "移除标签 角色" }));
    expect(onRemoveTag).toHaveBeenCalledWith("角色", [1, 2]);
  });

  it("reloads selected tags when tagRefreshVersion changes", async () => {
    mockedGetAssetTags.mockResolvedValueOnce(["旧标签"]).mockResolvedValueOnce(["新标签"]);
    const { rerender } = render(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );
    expect(await screen.findByText("旧标签")).toBeInTheDocument();
    rerender(
      <DetailsPanel
        selectedAssets={[makeAsset(1)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={1}
      />
    );
    expect(await screen.findByText("新标签")).toBeInTheDocument();
  });

  it("reloads common tags when same-length multi-selection changes", async () => {
    mockedListCommonTags.mockResolvedValueOnce(["角色"]).mockResolvedValueOnce(["特效"]);
    const { rerender } = render(
      <DetailsPanel
        selectedAssets={[makeAsset(1), makeAsset(2)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );
    expect(await screen.findByRole("button", { name: "移除标签 角色" })).toBeInTheDocument();
    rerender(
      <DetailsPanel
        selectedAssets={[makeAsset(1), makeAsset(3)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
        collections={[]}
        onAddToCollection={vi.fn()}
        activeCollectionId={null}
        onRemoveFromCollection={vi.fn()}
        onRemoveTag={vi.fn()}
        tagRefreshVersion={0}
      />
    );
    expect(await screen.findByRole("button", { name: "移除标签 特效" })).toBeInTheDocument();
    expect(mockedListCommonTags).toHaveBeenLastCalledWith([1, 3]);
  });
});
