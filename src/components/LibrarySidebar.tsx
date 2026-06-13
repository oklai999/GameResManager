import { useState } from "react";
import {
  AlertTriangle,
  Box,
  FileImage,
  Film,
  Folder,
  FolderOpen,
  Heart,
  List,
  Music,
  Plus,
  RotateCw,
  Tag as TagIcon,
  Trash2,
  Type,
  X,
} from "lucide-react";
import { CollectionManager } from "./CollectionManager";
import { TagManager } from "./TagManager";
import type { WorkbenchSection } from "./NavigationRail";
import type { Collection, FolderAssetCounts, LibraryFolder, ScanJob, Tag } from "../types/asset";

type Props = {
  folders: LibraryFolder[];
  collections: Collection[];
  tags: Tag[];
  activeFilter: string;
  activeSection: WorkbenchSection;
  hidden: boolean;
  selectedFolderId: number | null;
  selectedCollectionId: number | null;
  onFilterChange: (filter: string) => void;
  onSelectFolder: (folderId: number | null) => void;
  onSelectCollection: (collectionId: number | null) => void;
  onPickFolder: () => void;
  onScanFolder: (folderId: number) => void;
  onCancelScan: (jobId: number) => void;
  onDeleteFolder: (folderId: number) => void;
  onOpenFolder: (folder: LibraryFolder) => void;
  onCreateCollection: (name: string) => void;
  onUpdateCollection: (collectionId: number, name: string, description: string) => void;
  onDeleteCollection: (collectionId: number) => void;
  onUpdateTag: (tagId: number, name: string, color: string) => void;
  onDeleteTag: (tagId: number) => void;
  isScanning: boolean;
  latestJobs: Record<number, ScanJob | null>;
  folderCounts: Record<number, FolderAssetCounts>;
  settingsPanel?: React.ReactNode;
};

function formatDateTime(iso: string | null): string {
  if (!iso) return "未扫描";
  try {
    const d = new Date(iso);
    return d.toLocaleString("zh-CN");
  } catch {
    return iso;
  }
}

export function LibrarySidebar({
  folders, collections, tags, activeFilter, activeSection, hidden,
  selectedFolderId, selectedCollectionId,
  onFilterChange, onSelectFolder, onSelectCollection, onPickFolder, onScanFolder,
  onCancelScan, onDeleteFolder, onOpenFolder, onCreateCollection, onUpdateCollection, onDeleteCollection,
  onUpdateTag, onDeleteTag, isScanning, latestJobs, folderCounts, settingsPanel,
}: Props) {
  const [collectionName, setCollectionName] = useState("");
  const [isFolderManagerOpen, setIsFolderManagerOpen] = useState(false);
  const [isCollectionManagerOpen, setIsCollectionManagerOpen] = useState(false);
  const [isTagManagerOpen, setIsTagManagerOpen] = useState(false);

  if (hidden) {
    return <aside className="sidebar contextual-sidebar collapsed" aria-hidden="true" />;
  }

  const libraryFilters = [
    { id: "all", label: "全部资源", icon: List },
    { id: "favorites", label: "收藏", icon: Heart },
    { id: "missing", label: "缺失文件", icon: AlertTriangle },
  ];

  const typeFilters = [
    { id: "image", label: "图片", icon: FileImage },
    { id: "audio", label: "音频", icon: Music },
    { id: "video", label: "视频", icon: Film },
    { id: "font", label: "字体", icon: Type },
    { id: "model3d", label: "3D", icon: Box },
    { id: "spine", label: "Spine", icon: Box },
  ];

  const handleCreateCollection = (e: React.FormEvent) => {
    e.preventDefault();
    const name = collectionName.trim();
    if (name) {
      onCreateCollection(name);
      setCollectionName("");
    }
  };

  const confirmRemoveFolderIndex = (folder: LibraryFolder) => {
    if (window.confirm(`确定要从资源库移除「${folder.name}」的索引吗？这只会删除应用内索引和关联整理数据，不会删除磁盘上的原始文件。`)) {
      onDeleteFolder(folder.id);
    }
  };

  const renderNavFilters = (items: Array<{ id: string; label: string; icon: React.ElementType }>) => (
    <nav className="nav-list">
      {items.map(({ id, label, icon: Icon }) => (
        <button
          key={id}
          className={activeFilter === id ? "nav-item active" : "nav-item"}
          onClick={() => onFilterChange(id)}
        >
          <Icon size={15} aria-hidden="true" />
          <span>{label}</span>
        </button>
      ))}
    </nav>
  );

  const renderContent = () => {
    switch (activeSection) {
      case "library":
        return (
          <>
            <h1>资源库</h1>
            {renderNavFilters(libraryFilters)}
            <div className="panel-heading secondary folder-section-heading">
              <span>素材文件夹</span>
              <button
                type="button"
                className="folder-manager-open-btn"
                onClick={() => setIsFolderManagerOpen(true)}
                aria-label="管理文件夹"
              >
                管理
              </button>
            </div>
            <div className="folder-form">
              <button onClick={onPickFolder} className="nav-item" style={{ width: "100%" }}>
                <Plus size={15} aria-hidden="true" />
                <span>添加文件夹</span>
              </button>
            </div>
            <div className="folder-list">
              {folders.map((folder) => {
                const job = latestJobs[folder.id];
                const isRunning = job?.status === "running";
                const isSelected = selectedFolderId === folder.id;
                const counts = folderCounts[folder.id];
                return (
                  <div
                    className={isSelected ? "folder-row active" : "folder-row"}
                    key={folder.id}
                    title={folder.path}
                    onClick={() => onSelectFolder(isSelected ? null : folder.id)}
                  >
                    <div className="folder-row-header">
                      <span className="folder-name"><Folder size={14} aria-hidden="true" />{folder.name}</span>
                      <div className="folder-row-actions">
                        <button
                          className="open-folder-btn"
                          onClick={(e) => { e.stopPropagation(); onOpenFolder(folder); }}
                          aria-label={`打开文件夹 ${folder.name}`}
                        >
                          <FolderOpen size={13} aria-hidden="true" />
                        </button>
                        {isRunning ? (
                          <button
                            className="scan-btn cancel"
                            onClick={(e) => { e.stopPropagation(); onCancelScan(job!.id); }}
                            title="取消"
                          >
                            取消
                          </button>
                        ) : (
                          <button
                            className="scan-btn"
                            onClick={(e) => { e.stopPropagation(); onScanFolder(folder.id); }}
                            aria-label={`扫描 ${folder.name}`}
                            disabled={isScanning}
                          >
                            <RotateCw size={13} aria-hidden="true" />
                          </button>
                        )}
                        <button
                          className="delete-folder-btn"
                          onClick={(e) => {
                            e.stopPropagation();
                            confirmRemoveFolderIndex(folder);
                          }}
                          aria-label="从资源库移除索引"
                          title="从资源库移除索引"
                          disabled={isRunning}
                        >
                          <Trash2 size={13} aria-hidden="true" />
                        </button>
                      </div>
                    </div>
                    <div className="folder-row-meta">
                      {counts ? `${counts.total} 个资源` : "0 个资源"}
                      {counts && counts.missing > 0 ? ` · ${counts.missing} 个缺失` : ""}
                      {counts && !counts.is_accessible ? " · 路径不可访问" : ""}
                      {folder.last_scanned_at ? ` · ${formatDateTime(folder.last_scanned_at)}` : " · 未扫描"}
                    </div>
                  </div>
                );
              })}
            </div>
            {isFolderManagerOpen && (
              <div className="folder-manager-backdrop" role="presentation">
                <section
                  className="folder-manager-panel"
                  role="dialog"
                  aria-modal="true"
                  aria-label="资源库文件夹管理"
                >
                  <div className="folder-manager-header">
                    <div>
                      <h2>资源库文件夹管理</h2>
                      <p>备份工程前，可以在这里移除应用内索引。不会删除、移动或修改磁盘上的原始文件。</p>
                    </div>
                    <button
                      type="button"
                      className="folder-manager-close-btn"
                      onClick={() => setIsFolderManagerOpen(false)}
                      aria-label="关闭文件夹管理"
                    >
                      <X size={16} aria-hidden="true" />
                    </button>
                  </div>
                  {folders.length === 0 ? (
                    <div className="folder-manager-empty">还没有添加素材文件夹。</div>
                  ) : (
                    <div className="folder-manager-list">
                      {folders.map((folder) => {
                        const job = latestJobs[folder.id];
                        const isRunning = job?.status === "running";
                        const counts = folderCounts[folder.id];
                        return (
                          <article className="folder-manager-item" key={folder.id}>
                            <div className="folder-manager-info">
                              <div className="folder-manager-name">
                                <Folder size={15} aria-hidden="true" />
                                <span>{folder.name}</span>
                              </div>
                              <div className="folder-manager-path">{folder.path}</div>
                              <div className="folder-manager-meta">
                                {counts ? `${counts.total} 个资源` : "0 个资源"}
                                {counts && counts.missing > 0 ? ` · ${counts.missing} 个缺失` : ""}
                                {counts && !counts.is_accessible ? " · 路径不可访问" : ""}
                                {folder.last_scanned_at ? ` · ${formatDateTime(folder.last_scanned_at)}` : " · 未扫描"}
                              </div>
                              {isRunning && (
                                <div className="folder-manager-warning">扫描中，先取消扫描后再移除索引</div>
                              )}
                            </div>
                            <div className="folder-manager-actions">
                              <button
                                type="button"
                                className="open-folder-btn"
                                onClick={() => onOpenFolder(folder)}
                              >
                                打开文件夹
                              </button>
                              {isRunning ? (
                                <button
                                  type="button"
                                  className="scan-btn cancel"
                                  onClick={() => onCancelScan(job!.id)}
                                >
                                  取消扫描
                                </button>
                              ) : (
                                <button
                                  type="button"
                                  className="scan-btn"
                                  onClick={() => onScanFolder(folder.id)}
                                  disabled={isScanning}
                                >
                                  扫描
                                </button>
                              )}
                              <button
                                type="button"
                                className="remove-folder-index-btn"
                                onClick={() => confirmRemoveFolderIndex(folder)}
                                disabled={isRunning}
                                aria-label={`从资源库移除 ${folder.name} 的索引`}
                              >
                                从资源库移除
                              </button>
                            </div>
                          </article>
                        );
                      })}
                    </div>
                  )}
                </section>
              </div>
            )}
          </>
        );
      case "types":
        return (
          <>
            <h1>文件类型</h1>
            {renderNavFilters(typeFilters)}
          </>
        );
      case "tags":
        return (
          <>
            <h1>标签</h1>
            <div className="panel-heading secondary folder-section-heading">
              <span>标签</span>
              <button
                type="button"
                className="folder-manager-open-btn"
                onClick={() => setIsTagManagerOpen(true)}
                aria-label="管理标签"
              >
                管理
              </button>
            </div>
            <div className="folder-list">
              {tags.map((tag) => (
                <div
                  className="folder-row"
                  key={tag.id}
                  title={tag.name}
                >
                  <span className="folder-name">
                    <TagIcon size={14} aria-hidden="true" />
                    {tag.name}
                  </span>
                  <span className="collection-count">{tag.asset_count}</span>
                </div>
              ))}
            </div>
            {isTagManagerOpen && (
              <TagManager
                tags={tags}
                onClose={() => setIsTagManagerOpen(false)}
                onUpdate={onUpdateTag}
                onDelete={onDeleteTag}
              />
            )}
          </>
        );
      case "collections":
        return (
          <>
            <h1>集合</h1>
            <div className="panel-heading secondary folder-section-heading">
              <span>集合</span>
              <button
                type="button"
                className="folder-manager-open-btn"
                onClick={() => setIsCollectionManagerOpen(true)}
                aria-label="管理集合"
              >
                管理
              </button>
            </div>
            <form className="folder-form" onSubmit={handleCreateCollection}>
              <input
                type="text"
                value={collectionName}
                onChange={(e) => setCollectionName(e.target.value)}
                placeholder="新建集合..."
              />
              <button type="submit">创建</button>
            </form>
            <div className="folder-list">
              {collections.map((col) => {
                const isSelected = selectedCollectionId === col.id;
                return (
                  <div
                    className={isSelected ? "folder-row active" : "folder-row"}
                    key={col.id}
                    title={col.description || col.name}
                    onClick={() => onSelectCollection(isSelected ? null : col.id)}
                  >
                    <span className="folder-name"><List size={14} aria-hidden="true" />{col.name}</span>
                    <span className="collection-count">{col.asset_count}</span>
                  </div>
                );
              })}
            </div>
            {isCollectionManagerOpen && (
              <CollectionManager
                collections={collections}
                onClose={() => setIsCollectionManagerOpen(false)}
                onUpdate={onUpdateCollection}
                onDelete={onDeleteCollection}
              />
            )}
          </>
        );
      case "recent":
        return (
          <>
            <h1>最近使用</h1>
            <p className="recent-description">最近打开或查看的资源将显示在这里。</p>
            <button
              className={activeFilter === "recent" ? "nav-item active" : "nav-item"}
              onClick={() => onFilterChange("recent")}
            >
              <RotateCw size={15} aria-hidden="true" />
              <span>最近使用</span>
            </button>
          </>
        );
      case "settings":
        return <>{settingsPanel}</>;
    }
  };

  return (
    <aside className="sidebar">
      {renderContent()}
    </aside>
  );
}
