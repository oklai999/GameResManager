import type { Asset } from "../types/asset";

type Props = {
  selectedAssets: Asset[];
  onOpenFile: (asset: Asset) => void;
  onReveal: (asset: Asset) => void;
  onCopyPath: (asset: Asset) => void;
  onApplyTag: (tagName: string, assetIds: number[]) => void;
  onToggleFavorite: (asset: Asset) => void;
};

export function DetailsPanel({ selectedAssets, onOpenFile, onReveal, onCopyPath, onApplyTag, onToggleFavorite }: Props) {
  if (selectedAssets.length === 0) {
    return <aside className="details-panel muted">选择资源查看详情</aside>;
  }

  if (selectedAssets.length > 1) {
    const ids = selectedAssets.map((asset) => asset.id);
    return (
      <aside className="details-panel">
        <div className="panel-heading">已选择 {selectedAssets.length} 个资源</div>
        <button onClick={() => onApplyTag("待整理", ids)}>批量添加标签：待整理</button>
      </aside>
    );
  }

  const asset = selectedAssets[0];
  return (
    <aside className="details-panel">
      <div className="panel-heading">{asset.file_name}</div>
      <div className="detail-row">类型：{asset.asset_type}</div>
      <div className="detail-row">大小：{asset.file_size} bytes</div>
      <div className="detail-row path">{asset.absolute_path}</div>
      {asset.tags && asset.tags.length > 0 && (
        <div className="detail-row">标签：{asset.tags.join(", ")}</div>
      )}
      <button onClick={() => onToggleFavorite(asset)}>
        {asset.is_favorite ? "取消收藏" : "收藏"}
      </button>
      <button onClick={() => onOpenFile(asset)}>打开文件</button>
      <button onClick={() => onReveal(asset)}>打开所在目录</button>
      <button onClick={() => onCopyPath(asset)}>复制路径</button>
      <button onClick={() => onApplyTag("待整理", [asset.id])}>添加标签：待整理</button>
    </aside>
  );
}
