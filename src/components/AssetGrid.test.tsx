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

  it("shows no preview for image assets without a thumbnail", () => {
    render(
      <AssetGrid
        assets={[asset]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
      />
    );

    expect(screen.getByText("无预览")).toBeInTheDocument();
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
});
