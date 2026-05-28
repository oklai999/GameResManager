import { convertFileSrc } from "@tauri-apps/api/core";
import type { Asset } from "../types/asset";

type Props = {
  assets: Asset[];
  selectedIds: number[];
  onSelectionChange: (ids: number[]) => void;
  onToggleFavorite: (asset: Asset) => void;
};

export function AssetGrid({ assets, selectedIds, onSelectionChange, onToggleFavorite }: Props) {
  function toggle(assetId: number) {
    onSelectionChange(
      selectedIds.includes(assetId)
        ? selectedIds.filter((id) => id !== assetId)
        : [...selectedIds, assetId]
    );
  }

  return (
    <section className="asset-grid">
      {assets.map((asset) => (
        <div
          key={asset.id}
          className={selectedIds.includes(asset.id) ? "asset-card selected" : "asset-card"}
          onClick={() => toggle(asset.id)}
          title={asset.absolute_path}
          role="button"
          tabIndex={0}
        >
          <div className="thumb">
            {asset.thumbnail_path ? (
              <img src={convertFileSrc(asset.thumbnail_path)} alt="" />
            ) : (
              <span>{asset.extension.toUpperCase()}</span>
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
      ))}
    </section>
  );
}
