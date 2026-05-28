import { useState } from "react";
import type { LibraryFolder, ScanJob } from "../types/asset";

type Props = {
  folders: LibraryFolder[];
  activeFilter: string;
  onFilterChange: (filter: string) => void;
  onAddFolder: (name: string, path: string) => void;
  onScanFolder: (folderId: number) => void;
  onCancelScan: (jobId: number) => void;
  latestJobs: Record<number, ScanJob | null>;
  error: string | null;
};

export function LibrarySidebar({ folders, activeFilter, onFilterChange, onAddFolder, onScanFolder, onCancelScan, latestJobs, error }: Props) {
  const [name, setName] = useState("");
  const [path, setPath] = useState("");

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

  return (
    <aside className="sidebar">
      <div className="panel-heading">资源库</div>
      <nav className="nav-list">
        {filters.map((filter) => (
          <button
            key={filter.id}
            className={activeFilter === filter.id ? "nav-item active" : "nav-item"}
            onClick={() => onFilterChange(filter.id)}
          >
            {filter.label}
          </button>
        ))}
      </nav>

      <div className="panel-heading secondary">添加素材文件夹</div>
      <div className="folder-form">
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="名称"
        />
        <input
          value={path}
          onChange={(e) => setPath(e.target.value)}
          placeholder="绝对路径"
        />
        <button onClick={() => { onAddFolder(name, path); setName(""); setPath(""); }}>
          添加
        </button>
        {error && <div className="error-text">{error}</div>}
      </div>

      <div className="panel-heading secondary">素材文件夹</div>
      <div className="folder-list">
        {folders.map((folder) => {
          const job = latestJobs[folder.id];
          const isRunning = job?.status === "running";
          return (
            <div className="folder-row" key={folder.id} title={folder.path}>
              <span className="folder-name">{folder.name}</span>
              {isRunning ? (
                <button
                  className="scan-btn cancel"
                  onClick={() => onCancelScan(job!.id)}
                  title="取消"
                >
                  取消
                </button>
              ) : (
                <button
                  className="scan-btn"
                  onClick={() => onScanFolder(folder.id)}
                  title="扫描"
                >
                  扫描
                </button>
              )}
            </div>
          );
        })}
      </div>
    </aside>
  );
}
