import type { Asset } from "../types/asset";

type Props = {
  assets: Asset[];
  selectedIds: number[];
  onSelectionChange: (ids: number[]) => void;
};

export function AssetGrid({ assets, selectedIds, onSelectionChange }: Props) {
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
        <button
          key={asset.id}
          className={selectedIds.includes(asset.id) ? "asset-card selected" : "asset-card"}
          onClick={() => toggle(asset.id)}
          title={asset.absolute_path}
        >
          <div className="thumb">
            {asset.thumbnail_path ? <img src={asset.thumbnail_path} alt="" /> : <span>{asset.extension.toUpperCase()}</span>}
          </div>
          <div className="asset-name">{asset.file_name}</div>
          <div className="asset-meta">
            {asset.width && asset.height ? `${asset.width}x${asset.height}` : asset.asset_type}
          </div>
        </button>
      ))}
    </section>
  );
}
