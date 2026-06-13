import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
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

function mockGridMeasurements({ width, height }: { width: number; height: number }) {
  vi.stubGlobal(
    "ResizeObserver",
    class {
      callback: ResizeObserverCallback;
      constructor(callback: ResizeObserverCallback) {
        this.callback = callback;
      }
      observe(target: Element) {
        this.callback(
          [
            {
              target,
              contentRect: { width, height, top: 0, left: 0, bottom: height, right: width, x: 0, y: 0 },
              borderBoxSize: [{ inlineSize: width, blockSize: height }],
              contentBoxSize: [{ inlineSize: width, blockSize: height }],
              devicePixelContentBoxSize: [{ inlineSize: width, blockSize: height }],
            } as unknown as ResizeObserverEntry,
          ],
          this
        );
      }
      unobserve() {}
      disconnect() {}
    }
  );
}

describe("AssetGrid", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("calls selection change when an asset is clicked", async () => {
    const onSelectionChange = vi.fn();
    render(<AssetGrid assets={[asset]} selectedIds={[]} onSelectionChange={onSelectionChange} onToggleFavorite={vi.fn()} density="comfortable" />);

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
        density="comfortable"
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
        density="comfortable"
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
        density="comfortable"
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
        density="comfortable"
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
        density="comfortable"
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
        density="comfortable"
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
        density="comfortable"
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
        density="comfortable"
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
        density="comfortable"
      />
    );

    await userEvent.click(screen.getByRole("checkbox", { name: "选择 tree.png" }));

    expect(onSelectionChange).toHaveBeenCalledWith([1]);
  });

  it("retries thumbnail when thumbnail_path changes after a previous failure", () => {
    const { container, rerender } = render(
      <AssetGrid
        assets={[{ ...asset, thumbnail_path: "C:/cache/icon.webp", thumbnail_status: "ready" as const }]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
      />
    );

    const img = container.querySelector("img");
    expect(img).not.toBeNull();
    fireEvent.error(img as HTMLImageElement);

    expect(screen.getByText("预览加载失败")).toBeInTheDocument();

    rerender(
      <AssetGrid
        assets={[{ ...asset, thumbnail_path: "C:/cache/icon2.webp", thumbnail_status: "ready" as const }]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
      />
    );

    const newImg = container.querySelector("img");
    expect(newImg).not.toBeNull();
    expect(newImg).toHaveAttribute("src", "C:/cache/icon2.webp");
  });

  it("preserves thumbnail failure state after asset card is unmounted and remounted", () => {
    const { container, rerender } = render(
      <AssetGrid
        assets={[{ ...asset, thumbnail_path: "C:/cache/icon.webp", thumbnail_status: "ready" as const }]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
      />
    );

    const img = container.querySelector("img");
    expect(img).not.toBeNull();
    fireEvent.error(img as HTMLImageElement);

    expect(screen.getByText("预览加载失败")).toBeInTheDocument();

    // Unmount by removing asset
    rerender(
      <AssetGrid
        assets={[]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
      />
    );
    expect(container.querySelector("img")).toBeNull();

    // Remount by adding asset back
    rerender(
      <AssetGrid
        assets={[{ ...asset, thumbnail_path: "C:/cache/icon.webp", thumbnail_status: "ready" as const }]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
      />
    );

    const newImg = container.querySelector("img");
    expect(newImg).toBeNull();
    expect(screen.getByText("预览加载失败")).toBeInTheDocument();
  });

  it("renders only a visible window for large asset lists", () => {
    mockGridMeasurements({ width: 900, height: 600 });
    const manyAssets = Array.from({ length: 500 }, (_, index) =>
      makeAsset({
        id: index + 1,
        file_name: `asset-${index + 1}.png`,
        absolute_path: `C:/assets/asset-${index + 1}.png`,
      })
    );

    render(
      <AssetGrid
        assets={manyAssets}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
      />
    );

    expect(screen.getAllByRole("button", { name: /asset-/ }).length).toBeLessThan(100);
    expect(screen.getByText("asset-1.png")).toBeInTheDocument();
  });

  it("updates rendered window after scrolling", () => {
    mockGridMeasurements({ width: 900, height: 600 });
    const manyAssets = Array.from({ length: 500 }, (_, index) =>
      makeAsset({
        id: index + 1,
        file_name: `asset-${index + 1}.png`,
        absolute_path: `C:/assets/asset-${index + 1}.png`,
      })
    );

    const { container } = render(
      <AssetGrid
        assets={manyAssets}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
      />
    );

    expect(screen.getByText("asset-1.png")).toBeInTheDocument();

    const viewport = container.querySelector(".asset-grid-viewport") as HTMLElement;
    viewport.scrollTop = 2000;
    fireEvent.scroll(viewport);

    expect(screen.queryByText("asset-1.png")).not.toBeInTheDocument();
    expect(screen.getByText("asset-25.png")).toBeInTheDocument();
  });

  it("recovers from deep scroll when the list shortens", () => {
    mockGridMeasurements({ width: 900, height: 600 });
    const manyAssets = Array.from({ length: 500 }, (_, index) =>
      makeAsset({
        id: index + 1,
        file_name: `asset-${index + 1}.png`,
        absolute_path: `C:/assets/asset-${index + 1}.png`,
      })
    );

    const { container, rerender } = render(
      <AssetGrid
        assets={manyAssets}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
      />
    );

    const viewport = container.querySelector(".asset-grid-viewport") as HTMLElement;
    viewport.scrollTop = 10000;
    fireEvent.scroll(viewport);

    rerender(
      <AssetGrid
        assets={manyAssets.slice(0, 10)}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
      />
    );

    expect(viewport.scrollTop).toBeLessThan(10000);
    expect(screen.getByText("asset-1.png")).toBeInTheDocument();
  });

  it("preserves interaction after scrolling", async () => {
    mockGridMeasurements({ width: 900, height: 600 });
    const manyAssets = Array.from({ length: 500 }, (_, index) =>
      makeAsset({
        id: index + 1,
        file_name: `asset-${index + 1}.png`,
        absolute_path: `C:/assets/asset-${index + 1}.png`,
      })
    );

    const onSelectionChange = vi.fn();
    const { container } = render(
      <AssetGrid
        assets={manyAssets}
        selectedIds={[]}
        onSelectionChange={onSelectionChange}
        onToggleFavorite={vi.fn()}
        density="comfortable"
      />
    );

    const viewport = container.querySelector(".asset-grid-viewport") as HTMLElement;
    viewport.scrollTop = 2000;
    fireEvent.scroll(viewport);

    await userEvent.click(screen.getByText("asset-25.png"));

    expect(onSelectionChange).toHaveBeenCalledWith([25]);
  });

  it("resets scroll when resetKey changes even with same first asset id", () => {
    mockGridMeasurements({ width: 900, height: 600 });
    const assets = [
      makeAsset({ id: 1, file_name: "a.png", absolute_path: "C:/assets/a.png" }),
      makeAsset({ id: 2, file_name: "b.png", absolute_path: "C:/assets/b.png" }),
    ];

    const { container, rerender } = render(
      <AssetGrid
        assets={assets}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
        resetKey="A"
      />
    );

    const viewport = container.querySelector(".asset-grid-viewport") as HTMLElement;
    viewport.scrollTop = 1000;
    fireEvent.scroll(viewport);

    rerender(
      <AssetGrid
        assets={assets}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
        resetKey="B"
      />
    );

    expect(viewport.scrollTop).toBe(0);
  });

  it("preserves scroll when more assets append with same resetKey", () => {
    mockGridMeasurements({ width: 900, height: 600 });
    const fewAssets = [makeAsset({ id: 1, file_name: "a.png", absolute_path: "C:/assets/a.png" })];
    const manyAssets = Array.from({ length: 50 }, (_, i) =>
      makeAsset({ id: i + 1, file_name: `asset-${i + 1}.png`, absolute_path: `C:/assets/asset-${i + 1}.png` })
    );

    const { container, rerender } = render(
      <AssetGrid
        assets={fewAssets}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
        resetKey="A"
      />
    );

    const viewport = container.querySelector(".asset-grid-viewport") as HTMLElement;
    viewport.scrollTop = 500;
    fireEvent.scroll(viewport);

    rerender(
      <AssetGrid
        assets={manyAssets}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
        resetKey="A"
      />
    );

    expect(viewport.scrollTop).toBe(500);
  });

  it("renders comfortable card dimensions when density is comfortable", () => {
    mockGridMeasurements({ width: 900, height: 600 });
    render(
      <AssetGrid
        assets={[makeAsset({ id: 1, file_name: "test.png" })]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="comfortable"
      />
    );

    const card = screen.getByTitle("C:/assets/icon.png");
    expect(card.style.width).toBe("176px");
    expect(card.style.height).toBe("210px");
  });

  it("renders compact card dimensions when density is compact", () => {
    mockGridMeasurements({ width: 900, height: 600 });
    render(
      <AssetGrid
        assets={[makeAsset({ id: 1, file_name: "test.png" })]}
        selectedIds={[]}
        onSelectionChange={vi.fn()}
        onToggleFavorite={vi.fn()}
        density="compact"
      />
    );

    const card = screen.getByTitle("C:/assets/icon.png");
    expect(card.style.width).toBe("152px");
    expect(card.style.height).toBe("178px");
  });
});
