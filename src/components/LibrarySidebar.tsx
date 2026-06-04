import { useState } from "react";
import type { Collection, FolderAssetCounts, LibraryFolder, ScanJob } from "../types/asset";

type Props = {
  folders: LibraryFolder[];
  collections: Collection[];
  activeFilter: string;
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
  folders, collections, activeFilter, selectedFolderId, selectedCollectionId,
  onFilterChange, onSelectFolder, onSelectCollection, onPickFolder, onScanFolder,
  onCancelScan, onDeleteFolder, onOpenFolder, onCreateCollection, isScanning, latestJobs, folderCounts, settingsPanel,
}: Props) {
  const [collectionName, setCollectionName] = useState("");
  const filters = [
    { id: "all", label: "全部资源" },
    { id: "favorites", label: "收藏" },
    { id: "missing", label: "缺失文件" },
    { id: "image", label: "图片" },
    { id: "audio", label: "音频" },
    { id: "video", label: "视频" },
    { id: "font", label: "字体" },
    { id: "model3d", label: "3D" },
    { id: "spine", label: "Spine" },
  ];

  const handleCreateCollection = (e: React.FormEvent) => {
    e.preventDefault();
    const name = collectionName.trim();
    if (name) {
      onCreateCollection(name);
      setCollectionName("");
    }
  };

  return (
    <aside className="sidebar">
      <div className="panel-heading">资源库</div>
      <nav className="nav-list">
        {filters.map((filter) => (
          <button
            key={filter.id}
            className={activeFilter === filter.id ? "nav-item active" : "nav-item"}
            onClick={() => {
              onFilterChange(filter.id);
              onSelectCollection(null);
            }}
          >
            {filter.label}
          </button>
        ))}
      </nav>

      <div className="panel-heading secondary">素材文件夹</div>
      <div className="folder-form">
        <button onClick={onPickFolder} className="nav-item" style={{ width: "100%" }}>
          添加文件夹
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
              onClick={() => {
                onSelectFolder(isSelected ? null : folder.id);
                onSelectCollection(null);
              }}
            >
              <div className="folder-row-header">
                <span className="folder-name">{folder.name}</span>
                <div className="folder-row-actions">
                  <button
                    className="open-folder-btn"
                    onClick={(e) => { e.stopPropagation(); onOpenFolder(folder); }}
                    title="打开文件夹"
                  >
                    打开
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
                      title="扫描"
                      disabled={isScanning}
                    >
                      扫描
                    </button>
                  )}
                  <button
                    className="delete-folder-btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      if (window.confirm(`确定要从资源库移除「${folder.name}」的索引吗？这只会删除应用内索引和关联整理数据，不会删除磁盘上的原始文件。`)) {
                        onDeleteFolder(folder.id);
                      }
                    }}
                    title="从资源库移除索引"
                    disabled={isRunning}
                  >
                    ✕
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

      <div className="panel-heading secondary">集合</div>
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
              onClick={() => {
                onSelectCollection(isSelected ? null : col.id);
                onSelectFolder(null);
              }}
            >
              <span className="folder-name">{col.name}</span>
            </div>
          );
        })}
      </div>
      {settingsPanel}
    </aside>
  );
}
