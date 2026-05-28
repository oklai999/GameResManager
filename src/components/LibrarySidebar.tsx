import type { LibraryFolder } from "../types/asset";

type Props = {
  folders: LibraryFolder[];
  activeFilter: string;
  onFilterChange: (filter: string) => void;
};

export function LibrarySidebar({ folders, activeFilter, onFilterChange }: Props) {
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
      <div className="panel-heading secondary">素材文件夹</div>
      <div className="folder-list">
        {folders.map((folder) => (
          <div className="folder-row" key={folder.id} title={folder.path}>
            {folder.name}
          </div>
        ))}
      </div>
    </aside>
  );
}
