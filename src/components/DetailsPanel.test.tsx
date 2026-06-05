import { render, screen, waitFor } from "@testing-library/react";
import userEvent, { UserEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { DetailsPanel } from "./DetailsPanel";
import type { Asset, Collection } from "../types/asset";

vi.mock("../api/tauri", () => ({
  getAssetTags: vi.fn().mockResolvedValue([]),
  listCommonTags: vi.fn().mockResolvedValue([]),
  listTags: vi.fn().mockResolvedValue([]),
  updateAssetNote: vi.fn().mockResolvedValue({}),
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
  return { id, name, description: "" };
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
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "复制路径" }));
    expect(onCopyPath).toHaveBeenCalledWith(asset);
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
});
