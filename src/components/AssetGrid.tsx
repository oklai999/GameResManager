import { useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Box, FileImage, FileText, Film, Music, Star, Type } from "lucide-react";
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
  if (asset.extension.toLowerCase() === "psd") return "PSD";
  switch (asset.asset_type) {
    case "image":
      return "图片";
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

function assetTypeLabel(asset: Asset): string {
  if (asset.extension) {
    if (asset.extension.toLowerCase() === "spine") return "Spine";
    return asset.extension.toUpperCase();
  }
  switch (asset.asset_type) {
    case "model3d":
      return "3D";
    case "spine":
      return "Spine";
    default:
      return asset.asset_type.toUpperCase();
  }
}

function assetTypeIcon(assetType: Asset["asset_type"]) {
  switch (assetType) {
    case "image":
      return <FileImage size={14} aria-hidden="true" />;
    case "audio":
      return <Music size={14} aria-hidden="true" />;
    case "video":
      return <Film size={14} aria-hidden="true" />;
    case "font":
      return <Type size={14} aria-hidden="true" />;
    case "model3d":
    case "spine":
      return <Box size={14} aria-hidden="true" />;
    default:
      return <FileText size={14} aria-hidden="true" />;
  }
}

export function AssetGrid({ assets, selectedIds, onSelectionChange, onToggleFavorite }: Props) {
  const [failedIds, setFailedIds] = useState<Set<number>>(new Set());

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
            onClick={() => selectOnly(asset.id)}
            title={asset.absolute_path}
            role="button"
            tabIndex={0}
          >
            <input
              type="checkbox"
              className="asset-select-checkbox"
              checked={selectedIds.includes(asset.id)}
              aria-label={`选择 ${asset.file_name}`}
              onClick={(event) => event.stopPropagation()}
              onChange={(event) => toggleMulti(asset.id, event.currentTarget.checked)}
            />
            <div className="thumb">
              <span className="asset-type-badge">
                {assetTypeIcon(asset.asset_type)}
                {assetTypeLabel(asset)}
              </span>
              {canRenderThumbnail ? (
                <img
                  src={convertFileSrc(thumbnailPath)}
                  alt=""
                  onError={() => markFailed(asset.id)}
                />
              ) : (
                <span data-testid="asset-placeholder">{placeholderText(asset, imageLoadFailed)}</span>
              )}
            </div>
            <div className="asset-card-body">
              <div className="asset-name">{asset.file_name}</div>
              <div className="asset-meta">
                {asset.width && asset.height ? `${asset.width} x ${asset.height}` : asset.asset_type}
              </div>
              {asset.tags && asset.tags.length > 0 && (
                <div className="asset-tag-row">
                  {asset.tags.slice(0, 2).map((tag) => (
                    <span key={tag} className="asset-tag-chip">{tag}</span>
                  ))}
                  {asset.tags.length > 2 && (
                    <span className="asset-tag-chip muted-chip">+{asset.tags.length - 2}</span>
                  )}
                </div>
              )}
            </div>
            <button
              className={asset.is_favorite ? "favorite active" : "favorite"}
              onClick={(e) => {
                e.stopPropagation();
                onToggleFavorite(asset);
              }}
              title={asset.is_favorite ? "取消收藏" : "收藏"}
            >
              <Star size={15} fill={asset.is_favorite ? "currentColor" : "none"} aria-hidden="true" />
            </button>
          </div>
        );
      })}
    </section>
  );
}
