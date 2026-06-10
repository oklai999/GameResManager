import { useEffect, useRef, useState } from "react";
import type { Asset } from "../types/asset";
import { AssetCard } from "./AssetCard";

const CARD_WIDTH = 176;
const CARD_HEIGHT = 238;
const GRID_GAP = 12;
const OVERSCAN_ROWS = 3;

type Props = {
  assets: Asset[];
  selectedIds: number[];
  onSelectionChange: (ids: number[]) => void;
  onToggleFavorite: (asset: Asset) => void;
  resetKey?: string | number;
};

export function AssetGrid({ assets, selectedIds, onSelectionChange, onToggleFavorite, resetKey }: Props) {
  const [failedMap, setFailedMap] = useState<Map<number, { thumbnail_path: string | null; thumbnail_status: string }>>(new Map());

  const viewportRef = useRef<HTMLElement | null>(null);
  const [viewport, setViewport] = useState({ width: 0, height: 0 });
  const [scrollTop, setScrollTop] = useState(0);

  useEffect(() => {
    const el = viewportRef.current;
    if (!el) return;

    function updateFromClient(element: HTMLElement) {
      setViewport({
        width: element.clientWidth,
        height: element.clientHeight,
      });
    }

    let ro: ResizeObserver | null = null;
    if (typeof ResizeObserver !== "undefined") {
      ro = new ResizeObserver((entries) => {
        const entry = entries[0];
        setViewport({
          width: entry.contentRect.width,
          height: entry.contentRect.height,
        });
      });
      ro.observe(el);
    } else {
      updateFromClient(el);
    }

    return () => {
      if (ro) ro.disconnect();
    };
  }, []);

  // Clear failed state when a currently-visible asset's thumbnail metadata changes.
  useEffect(() => {
    const assetMap = new Map(assets.map((a) => [a.id, a]));
    setFailedMap((prev) => {
      const next = new Map(prev);
      for (const [id, signature] of next) {
        const current = assetMap.get(id);
        if (
          current &&
          (current.thumbnail_path !== signature.thumbnail_path ||
            current.thumbnail_status !== signature.thumbnail_status)
        ) {
          next.delete(id);
        }
      }
      return next;
    });
  }, [assets]);

  function selectOnly(assetId: number) {
    onSelectionChange([assetId]);
  }

  function toggleMulti(assetId: number, checked: boolean) {
    onSelectionChange(
      checked
        ? [...selectedIds.filter((id) => id !== assetId), assetId]
        : selectedIds.filter((id) => id !== assetId)
    );
  }

  function markFailed(assetId: number) {
    const asset = assets.find((a) => a.id === assetId);
    if (!asset) return;
    setFailedMap((prev) => {
      const next = new Map(prev);
      next.set(assetId, {
        thumbnail_path: asset.thumbnail_path,
        thumbnail_status: asset.thumbnail_status,
      });
      return next;
    });
  }

  const columns = Math.max(1, Math.floor((viewport.width + GRID_GAP) / (CARD_WIDTH + GRID_GAP)));
  const rowCount = Math.ceil(assets.length / columns);

  // Reset scroll to top when the dataset is replaced (query/filter/sort change),
  // but not when appending more assets (resetKey stays the same).
  const prevResetKeyRef = useRef<string | number | undefined>(undefined);
  useEffect(() => {
    if (prevResetKeyRef.current !== undefined && prevResetKeyRef.current !== resetKey) {
      const el = viewportRef.current;
      if (el) {
        el.scrollTop = 0;
        setScrollTop(0);
      }
    }
    prevResetKeyRef.current = resetKey;
  }, [resetKey]);

  // Clamp scrollTop when the list shrinks so the viewport doesn't sit past the content.
  useEffect(() => {
    const el = viewportRef.current;
    if (!el) return;
    const totalHeight = rowCount * (CARD_HEIGHT + GRID_GAP);
    const maxScroll = Math.max(0, totalHeight - el.clientHeight);
    if (el.scrollTop > maxScroll) {
      el.scrollTop = maxScroll;
      setScrollTop(maxScroll);
    }
  }, [assets.length, rowCount]);

  const firstVisibleRow = Math.max(
    0,
    Math.min(
      Math.floor(scrollTop / (CARD_HEIGHT + GRID_GAP)) - OVERSCAN_ROWS,
      rowCount - 1
    )
  );
  const visibleRowCount = Math.ceil(viewport.height / (CARD_HEIGHT + GRID_GAP)) + OVERSCAN_ROWS * 2;
  const lastVisibleRow = Math.min(rowCount, firstVisibleRow + visibleRowCount);
  const startIndex = firstVisibleRow * columns;
  const endIndex = Math.min(assets.length, lastVisibleRow * columns);
  const visibleAssets = assets.slice(startIndex, endIndex);

  return (
    <section
      ref={viewportRef}
      className="asset-grid-viewport"
      onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
    >
      <div
        className="asset-grid-spacer"
        style={{ height: rowCount * (CARD_HEIGHT + GRID_GAP) }}
      >
        {visibleAssets.map((asset, localIndex) => {
          const index = startIndex + localIndex;
          const row = Math.floor(index / columns);
          const column = index % columns;
          const signature = failedMap.get(asset.id);
          const imageLoadFailed =
            signature !== undefined &&
            signature.thumbnail_path === asset.thumbnail_path &&
            signature.thumbnail_status === asset.thumbnail_status;

          return (
            <AssetCard
              key={asset.id}
              asset={asset}
              selected={selectedIds.includes(asset.id)}
              imageLoadFailed={imageLoadFailed}
              onImageError={markFailed}
              onSelectOnly={selectOnly}
              onToggleMulti={toggleMulti}
              onToggleFavorite={onToggleFavorite}
              style={{
                position: "absolute",
                width: CARD_WIDTH,
                height: CARD_HEIGHT,
                overflow: "hidden",
                transform: `translate(${column * (CARD_WIDTH + GRID_GAP)}px, ${row * (CARD_HEIGHT + GRID_GAP)}px)`,
              }}
            />
          );
        })}
      </div>
    </section>
  );
}
