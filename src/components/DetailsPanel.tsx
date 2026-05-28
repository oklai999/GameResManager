import type { Asset } from "../types/asset";

type Props = {
  selectedAssets: Asset[];
  onOpenFile: (asset: Asset) => void;
  onReveal: (asset: Asset) => void;
  onCopyPath: (asset: Asset) => void;
  onApplyTag: (tagName: string, assetIds: number[]) => void;
};

export function DetailsPanel({ selectedAssets, onOpenFile, onReveal, onCopyPath, onApplyTag }: Props) {
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
      <button onClick={() => onOpenFile(asset)}>打开文件</button>
      <button onClick={() => onReveal(asset)}>打开所在目录</button>
      <button onClick={() => onCopyPath(asset)}>复制路径</button>
    </aside>
  );
}
