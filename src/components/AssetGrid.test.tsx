import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { AssetGrid } from "./AssetGrid";
import type { Asset } from "../types/asset";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => path,
}));

const asset: Asset = {
  id: 1,
  library_folder_id: 1,
  absolute_path: "C:/assets/icon.png",
  file_name: "icon.png",
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

function makeAsset(overrides: Partial<Asset>): Asset {
  return { ...asset, ...overrides };
}

describe("AssetGrid", () => {
  it("calls selection change when an asset is clicked", async () => {
    const onSelectionChange = vi.fn();
    render(<AssetGrid assets={[asset]} selectedIds={[]} onSelectionChange={onSelectionChange} onToggleFavorite={vi.fn()} />);

    await userEvent.click(screen.getByTitle("C:/assets/icon.png"));

    expect(onSelectionChange).toHaveBeenCalledWith([1]);
  });

  it("shows thumbnail failure for failed thumbnail status", () => {
    render(
      <AssetGrid
        assets={[{ ...asset, thumbnail_status: "failed" as const, thumbnail_error: "Invalid PNG signature" }]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
      />
    );

    expect(screen.getByText("缩略图失败")).toBeInTheDocument();
  });

  it("shows image placeholder for image assets without a thumbnail", () => {
    render(
      <AssetGrid
        assets={[asset]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
      />
    );

    expect(screen.getByText("图片")).toBeInTheDocument();
  });

  it("shows asset type and tag chips for visual scanning", () => {
    render(
      <AssetGrid
        assets={[{ ...asset, tags: ["场景", "森林", "日照"] }]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
      />
    );

    expect(screen.getByText("PNG")).toBeInTheDocument();
    expect(screen.getByText("场景")).toBeInTheDocument();
    expect(screen.getByText("森林")).toBeInTheDocument();
    expect(screen.getByText("+1")).toBeInTheDocument();
  });

  it("shows preview load failure after a ready thumbnail image errors", () => {
    const { container } = render(
      <AssetGrid
        assets={[{ ...asset, thumbnail_path: "C:/cache/icon.webp", thumbnail_status: "ready" as const }]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
      />
    );

    const img = container.querySelector("img");
    expect(img).not.toBeNull();
    fireEvent.error(img as HTMLImageElement);

    expect(screen.getByText("预览加载失败")).toBeInTheDocument();
  });

  it("shows manageable placeholders for non-previewable asset types", () => {
    const { container } = render(
      <AssetGrid
        assets={[
          makeAsset({ id: 1, file_name: "concept.psd", extension: "psd", asset_type: "image", thumbnail_status: "none" }),
          makeAsset({ id: 2, file_name: "hero.spine", extension: "spine", asset_type: "spine", thumbnail_status: "none" }),
          makeAsset({ id: 3, file_name: "mesh.glb", extension: "glb", asset_type: "model3d", thumbnail_status: "none" }),
        ]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
      />
    );

    const placeholders = screen.getAllByTestId("asset-placeholder");
    const texts = placeholders.map((el) => el.textContent);
    expect(texts).toContain("PSD");
    expect(texts).toContain("Spine");
    expect(texts).toContain("3D");
  });

  it("single-selects an asset when the card is clicked", async () => {
    const onSelectionChange = vi.fn();
    render(
      <AssetGrid
        assets={[
          asset,
          makeAsset({ id: 2, file_name: "tree.png", absolute_path: "C:/assets/tree.png" }),
        ]}
        selectedIds={[2]}
        onSelectionChange={onSelectionChange}
        onToggleFavorite={vi.fn()}
      />
    );

    await userEvent.click(screen.getByTitle("C:/assets/icon.png"));

    expect(onSelectionChange).toHaveBeenCalledWith([1]);
  });

  it("keeps the selected card selected when it is clicked again", async () => {
    const onSelectionChange = vi.fn();
    render(
      <AssetGrid
        assets={[asset]}
        selectedIds={[1]}
        onSelectionChange={onSelectionChange}
        onToggleFavorite={vi.fn()}
      />
    );

    await userEvent.click(screen.getByTitle("C:/assets/icon.png"));

    expect(onSelectionChange).toHaveBeenCalledWith([1]);
  });

  it("uses card checkboxes for multi-select", async () => {
    const onSelectionChange = vi.fn();
    render(
      <AssetGrid
        assets={[
          asset,
          makeAsset({ id: 2, file_name: "tree.png", absolute_path: "C:/assets/tree.png" }),
        ]}
        selectedIds={[1]}
        onSelectionChange={onSelectionChange}
        onToggleFavorite={vi.fn()}
      />
    );

    await userEvent.click(screen.getByRole("checkbox", { name: "选择 tree.png" }));

    expect(onSelectionChange).toHaveBeenCalledWith([1, 2]);
  });

  it("removes an asset from multi-select when its checkbox is unchecked", async () => {
    const onSelectionChange = vi.fn();
    render(
      <AssetGrid
        assets={[
          asset,
          makeAsset({ id: 2, file_name: "tree.png", absolute_path: "C:/assets/tree.png" }),
        ]}
        selectedIds={[1, 2]}
        onSelectionChange={onSelectionChange}
        onToggleFavorite={vi.fn()}
      />
    );

    await userEvent.click(screen.getByRole("checkbox", { name: "选择 tree.png" }));

    expect(onSelectionChange).toHaveBeenCalledWith([1]);
  });
});
