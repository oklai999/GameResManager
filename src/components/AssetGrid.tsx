import { useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { Asset } from "../types/asset";

type Props = {
  assets: Asset[];
  selectedIds: number[];
  onSelectionChange: (ids: number[]) => void;
  onToggleFavorite: (asset: Asset) => void;
};

function placeholderText(asset: Asset, imageLoadFailed: boolean): string {
  if (asset.is_missing) return "缺失";
  if (imageLoadFailed) return "预览加载失败";
  if (asset.thumbnail_status === "failed") return "缩略图失败";
  if (asset.thumbnail_status === "queued" || asset.thumbnail_status === "generating") return "生成中";
  switch (asset.asset_type) {
    case "image":
      return "无预览";
    case "audio":
      return "音频";
    case "video":
      return "视频";
    case "font":
      return "字体";
    case "model3d":
      return "3D";
    case "spine":
      return "Spine";
    default:
      return asset.extension.toUpperCase();
  }
}

export function AssetGrid({ assets, selectedIds, onSelectionChange, onToggleFavorite }: Props) {
  const [failedIds, setFailedIds] = useState<Set<number>>(new Set());

  function toggle(assetId: number) {
    onSelectionChange(
      selectedIds.includes(assetId)
        ? selectedIds.filter((id) => id !== assetId)
        : [...selectedIds, assetId]
    );
  }

  function markFailed(assetId: number) {
    setFailedIds((prev) => new Set(prev).add(assetId));
  }

  return (
    <section className="asset-grid">
      {assets.map((asset) => {
        const imageLoadFailed = failedIds.has(asset.id);
        const thumbnailPath = asset.thumbnail_path;
        const canRenderThumbnail =
          !asset.is_missing &&
          asset.thumbnail_status === "ready" &&
          typeof thumbnailPath === "string" &&
          thumbnailPath.length > 0 &&
          !imageLoadFailed;

        return (
          <div
            key={asset.id}
            className={selectedIds.includes(asset.id) ? "asset-card selected" : "asset-card"}
            onClick={() => toggle(asset.id)}
            title={asset.absolute_path}
            role="button"
            tabIndex={0}
          >
            <div className="thumb">
              {canRenderThumbnail ? (
                <img
                  src={convertFileSrc(thumbnailPath)}
                  alt=""
                  onError={() => markFailed(asset.id)}
                />
              ) : (
                <span>{placeholderText(asset, imageLoadFailed)}</span>
              )}
            </div>
            <div className="asset-name">{asset.file_name}</div>
            <div className="asset-meta">
              {asset.width && asset.height ? `${asset.width}x${asset.height}` : asset.asset_type}
            </div>
            <button
              className={asset.is_favorite ? "favorite active" : "favorite"}
              onClick={(e) => {
                e.stopPropagation();
                onToggleFavorite(asset);
              }}
              title={asset.is_favorite ? "取消收藏" : "收藏"}
            >
              {asset.is_favorite ? "★" : "☆"}
            </button>
          </div>
        );
      })}
    </section>
  );
}
