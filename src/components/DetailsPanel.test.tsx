import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DetailsPanel } from "./DetailsPanel";
import type { Asset } from "../types/asset";

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

describe("DetailsPanel", () => {
  it("shows batch mode for multiple selected assets", () => {
    render(
      <DetailsPanel
        selectedAssets={[makeAsset(1), makeAsset(2)]}
        onOpenFile={vi.fn()}
        onReveal={vi.fn()}
        onCopyPath={vi.fn()}
        onApplyTag={vi.fn()}
        onToggleFavorite={vi.fn()}
      />
    );

    expect(screen.getByText("已选择 2 个资源")).toBeInTheDocument();
  });
});
