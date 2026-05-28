import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { AssetGrid } from "./AssetGrid";
import type { Asset } from "../types/asset";

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
  note: "",
  is_favorite: false,
  is_missing: false,
  created_at: "2026-05-27T00:00:00Z",
  updated_at: "2026-05-27T00:00:00Z",
};

describe("AssetGrid", () => {
  it("calls selection change when an asset is clicked", async () => {
    const onSelectionChange = vi.fn();
    render(<AssetGrid assets={[asset]} selectedIds={[]} onSelectionChange={onSelectionChange} />);

    await userEvent.click(screen.getByTitle("C:/assets/icon.png"));

    expect(onSelectionChange).toHaveBeenCalledWith([1]);
  });
});
