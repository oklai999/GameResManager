import { useCallback, useEffect, useState } from "react";
import type { Asset, Collection } from "../types/asset";
import { getAssetTags, listCommonTags, updateAssetNote } from "../api/tauri";
import { EmptyState } from "./EmptyState";
import { TagEditor } from "./TagEditor";

type Props = {
  selectedAssets: Asset[];
  collections: Collection[];
  onOpenFile: (asset: Asset) => void;
  onReveal: (asset: Asset) => void;
  onCopyPath: (asset: Asset) => void;
  onApplyTag: (tagName: string, assetIds: number[]) => void;
  onToggleFavorite: (asset: Asset) => void;
  onAddToCollection: (collectionId: number, assetIds: number[]) => void;
  onUpdateNote?: (asset: Asset) => void;
};

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

function formatDateTime(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleString("zh-CN");
  } catch {
    return iso;
  }
}

export function DetailsPanel({
  selectedAssets,
  collections,
  onOpenFile,
  onReveal,
  onCopyPath,
  onApplyTag,
  onToggleFavorite,
  onAddToCollection,
  onUpdateNote,
}: Props) {
  const [tags, setTags] = useState<string[]>([]);
  const [note, setNote] = useState("");
  const [noteSaving, setNoteSaving] = useState(false);
  const [noteError, setNoteError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    if (selectedAssets.length === 0) {
      setTags([]);
      setNote("");
      setNoteError(null);
      return;
    }
    if (selectedAssets.length === 1) {
      getAssetTags(selectedAssets[0].id)
        .then((t) => { if (!cancelled) setTags(t); })
        .catch(() => { if (!cancelled) setTags([]); });
      setNote(selectedAssets[0].note ?? "");
      setNoteError(null);
    } else {
      const ids = selectedAssets.map((a) => a.id);
      listCommonTags(ids)
        .then((t) => { if (!cancelled) setTags(t); })
        .catch(() => { if (!cancelled) setTags([]); });
      setNote("");
      setNoteError(null);
    }
    return () => { cancelled = true; };
  }, [selectedAssets.length, selectedAssets[0]?.id, selectedAssets[0]?.note]);

  const saveNote = useCallback(async () => {
    if (noteSaving) return;
    if (selectedAssets.length !== 1) return;
    const asset = selectedAssets[0];
    if (note === (asset.note ?? "")) return;
    setNoteSaving(true);
    setNoteError(null);
    try {
      const updated = await updateAssetNote(asset.id, note);
      if (onUpdateNote) onUpdateNote(updated);
    } catch (e) {
      setNoteError(String(e));
    } finally {
      setNoteSaving(false);
    }
  }, [selectedAssets, note, onUpdateNote, noteSaving]);

  if (selectedAssets.length === 0) {
    return <aside className="details-panel muted">选择资源查看详情</aside>;
  }

  const ids = selectedAssets.map((asset) => asset.id);

  if (selectedAssets.length > 1) {
    return (
      <aside className="details-panel">
        <div className="panel-heading">已选择 {selectedAssets.length} 个资源</div>
        <div className="detail-row">共同标签：{tags.length > 0 ? tags.join(", ") : "无"}</div>
        <TagEditor existingTags={tags} onApply={(tagName) => onApplyTag(tagName, ids)} />
        {collections.length > 0 && (
          <div className="detail-actions">
            <div className="panel-heading secondary">添加到集合</div>
            <div className="collection-list">
              {collections.map((col) => (
                <button key={col.id} className="collection-btn" onClick={() => onAddToCollection(col.id, ids)}>
                  {col.name}
                </button>
              ))}
            </div>
          </div>
        )}
      </aside>
    );
  }

  const asset = selectedAssets[0];

  return (
    <aside className="details-panel">
      <div className="panel-heading">{asset.file_name}</div>
      {asset.is_missing ? (
        <EmptyState variant="file-missing" />
      ) : (
        <>
          <div className="detail-row">类型：{asset.asset_type}</div>
          <div className="detail-row">大小：{formatFileSize(asset.file_size)}</div>
          {asset.width && asset.height && (
            <div className="detail-row">尺寸：{asset.width} x {asset.height}</div>
          )}
          <div className="detail-row">修改时间：{formatDateTime(asset.modified_at)}</div>
          <div className="detail-row path">{asset.absolute_path}</div>
          {asset.thumbnail_status === "failed" && asset.thumbnail_error && (
            <div className="detail-row error-text">缩略图错误：{asset.thumbnail_error}</div>
          )}
        </>
      )}

      <div className="panel-heading secondary">备注</div>
      <textarea
        className="note-textarea"
        rows={4}
        value={note}
        onChange={(e) => setNote(e.target.value)}
        onBlur={saveNote}
        placeholder="输入备注..."
        disabled={noteSaving}
      />
      <div className="note-actions">
        <button className="note-save-btn" onClick={saveNote} disabled={noteSaving}>
          {noteSaving ? "保存中..." : "保存备注"}
        </button>
        {noteError && <span className="error-text">{noteError}</span>}
      </div>

      <TagEditor existingTags={tags} onApply={(tagName) => onApplyTag(tagName, [asset.id])} />

      {collections.length > 0 && (
        <div className="detail-actions">
          <div className="panel-heading secondary">添加到集合</div>
          <div className="collection-list">
            {collections.map((col) => (
              <button key={col.id} className="collection-btn" onClick={() => onAddToCollection(col.id, [asset.id])}>
                {col.name}
              </button>
            ))}
          </div>
        </div>
      )}

      <div className="detail-actions">
        <button onClick={() => onToggleFavorite(asset)}>
          {asset.is_favorite ? "取消收藏" : "收藏"}
        </button>
        <button onClick={() => onOpenFile(asset)}>打开文件</button>
        <button onClick={() => onReveal(asset)}>打开所在目录</button>
        <button onClick={() => onCopyPath(asset)}>复制路径</button>
      </div>
    </aside>
  );
}
