import { useCallback, useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Copy, ExternalLink, FolderOpen, Star } from "lucide-react";
import type { Asset, AssetPathVariants, Collection } from "../types/asset";
import { assetPathVariants, getAssetTags, listCommonTags, updateAssetNote } from "../api/tauri";
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
  recentTags?: string[];
  onCopyText?: (text: string) => void;
  projectRoot?: string;
  onProjectRootChange?: (root: string) => void;
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

function getFolderPath(absolutePath: string): string {
  const lastSep = Math.max(absolutePath.lastIndexOf("\\"), absolutePath.lastIndexOf("/"));
  return lastSep > 0 ? absolutePath.substring(0, lastSep) : "";
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
  recentTags = [],
  onCopyText,
  projectRoot,
  onProjectRootChange,
}: Props) {
  const [tags, setTags] = useState<string[]>([]);
  const [note, setNote] = useState("");
  const [noteSaving, setNoteSaving] = useState(false);
  const [noteError, setNoteError] = useState<string | null>(null);
  const [pathVariants, setPathVariants] = useState<AssetPathVariants | null>(null);

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

  useEffect(() => {
    let cancelled = false;
    if (selectedAssets.length !== 1) {
      setPathVariants(null);
      return;
    }
    const root = projectRoot?.trim() || null;
    if (!root) {
      setPathVariants(null);
      return;
    }
    setPathVariants(null);
    assetPathVariants(selectedAssets[0].absolute_path, root)
      .then((pv) => { if (!cancelled) setPathVariants(pv); })
      .catch(() => { if (!cancelled) setPathVariants(null); });
    return () => { cancelled = true; };
  }, [selectedAssets.length, selectedAssets[0]?.absolute_path, projectRoot]);

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
    return (
      <aside className="details-panel inspector-empty">
        <div className="panel-heading">详情</div>
        <EmptyState variant="no-selection" />
      </aside>
    );
  }

  const ids = selectedAssets.map((asset) => asset.id);

  if (selectedAssets.length > 1) {
    return (
      <aside className="details-panel">
        <div className="inspector-head">
          <div>
            <div className="panel-heading">批量整理</div>
            <div className="muted">已选择 {selectedAssets.length} 个资源</div>
          </div>
        </div>
        <section className="inspector-section">
          <div className="section-title">共同标签</div>
          <div className="detail-row">{tags.length > 0 ? tags.join(", ") : "无"}</div>
          <TagEditor existingTags={tags} recentTags={recentTags} onApply={(tagName) => onApplyTag(tagName, ids)} />
        </section>
        {collections.length > 0 && (
          <section className="inspector-section">
            <div className="section-title">添加到集合</div>
            <div className="collection-list">
              {collections.map((col) => (
                <button key={col.id} className="collection-btn" onClick={() => onAddToCollection(col.id, ids)}>
                  {col.name}
                </button>
              ))}
            </div>
          </section>
        )}
      </aside>
    );
  }

  const asset = selectedAssets[0];
  const canPreview =
    !asset.is_missing &&
    asset.thumbnail_status === "ready" &&
    typeof asset.thumbnail_path === "string" &&
    asset.thumbnail_path.length > 0;

  return (
    <aside className="details-panel">
      <div className="inspector-head">
        <div>
          <div className="panel-heading">{asset.file_name}</div>
          <div className="muted">{asset.extension.toUpperCase()} · {formatFileSize(asset.file_size)}</div>
        </div>
        <button className={asset.is_favorite ? "icon-action active" : "icon-action"} onClick={() => onToggleFavorite(asset)} title={asset.is_favorite ? "取消收藏" : "收藏"}>
          <Star size={17} fill={asset.is_favorite ? "currentColor" : "none"} aria-hidden="true" />
        </button>
      </div>
      <div className="detail-preview">
        {canPreview ? (
          <img src={convertFileSrc(asset.thumbnail_path!)} alt="" />
        ) : (
          <span>{asset.is_missing ? "文件缺失" : asset.asset_type}</span>
        )}
      </div>
      {asset.is_missing ? (
        <EmptyState variant="file-missing" />
      ) : (
        <section className="inspector-section">
          <div className="section-title">属性</div>
          <div className="detail-grid">
            <span>类型</span><strong>{asset.asset_type}</strong>
            <span>大小</span><strong>{formatFileSize(asset.file_size)}</strong>
          {asset.width && asset.height && (
              <><span>尺寸</span><strong>{asset.width} x {asset.height}</strong></>
          )}
            <span>修改时间</span><strong>{formatDateTime(asset.modified_at)}</strong>
          </div>
          <div className="detail-row path">{asset.absolute_path}</div>
          {onCopyText && (
            <>
              <div className="path-variant-actions">
                <button onClick={() => onCopyText(asset.absolute_path)}>复制绝对路径</button>
                <button onClick={() => onCopyText(asset.absolute_path.replace(/\\/g, "/"))}>复制正斜杠路径</button>
                <button onClick={() => onCopyText(getFolderPath(asset.absolute_path))}>复制文件夹路径</button>
                <button onClick={() => onCopyText(asset.file_name)}>复制文件名</button>
              </div>
              <div className="project-root-row">
                <input
                  className="tag-input"
                  type="text"
                  value={projectRoot ?? ""}
                  onChange={(e) => onProjectRootChange?.(e.target.value)}
                  placeholder="项目根目录（可选）"
                />
              </div>
              {projectRoot?.trim() && (
                <div className="path-variant-actions">
                  {pathVariants?.godot_res_path ? (
                    <button onClick={() => onCopyText(pathVariants.godot_res_path!)}>复制 res:// 路径</button>
                  ) : (
                    <span className="muted">不在项目根目录下</span>
                  )}
                </div>
              )}
            </>
          )}
          {asset.thumbnail_status === "failed" && asset.thumbnail_error && (
            <div className="detail-row error-text">缩略图错误：{asset.thumbnail_error}</div>
          )}
          {asset.thumbnail_status === "ready" && asset.thumbnail_path && (
            <div className="detail-row path">缩略图缓存：{asset.thumbnail_path}</div>
          )}
        </section>
      )}

      <section className="inspector-section">
        <div className="section-title">备注</div>
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
      </section>

      <section className="inspector-section">
        <div className="section-title">标签</div>
        <TagEditor existingTags={tags} recentTags={recentTags} onApply={(tagName) => onApplyTag(tagName, [asset.id])} />
      </section>

      {collections.length > 0 && (
        <section className="inspector-section">
          <div className="section-title">添加到集合</div>
          <div className="collection-list">
            {collections.map((col) => (
              <button key={col.id} className="collection-btn" onClick={() => onAddToCollection(col.id, [asset.id])}>
                {col.name}
              </button>
            ))}
          </div>
        </section>
      )}

      <section className="detail-actions inspector-section">
        <button onClick={() => onOpenFile(asset)}><ExternalLink size={15} aria-hidden="true" />打开文件</button>
        <button onClick={() => onReveal(asset)}><FolderOpen size={15} aria-hidden="true" />打开所在目录</button>
        <button onClick={() => onCopyPath(asset)}><Copy size={15} aria-hidden="true" />复制路径</button>
      </section>
    </aside>
  );
}
