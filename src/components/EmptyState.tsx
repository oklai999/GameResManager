import { AlertTriangle, CalendarX, FolderPlus, Search, SearchX } from "lucide-react";

type Props = {
  variant: "no-folders" | "no-assets" | "no-results" | "no-selection" | "file-missing" | "no-recent-activity" | "no-recent-filter-results";
};

const content: Record<Props["variant"], { title: string; description: string; icon: React.ElementType }> = {
  "no-folders": {
    title: "还没有添加素材文件夹",
    description: '点击左侧"添加文件夹"按钮，选择一个本地文件夹开始使用。',
    icon: FolderPlus,
  },
  "no-assets": {
    title: "还没有扫描资源",
    description: "选中一个素材文件夹，点击右侧的扫描按钮开始索引资源。",
    icon: Search,
  },
  "no-results": {
    title: "没有找到匹配的资源",
    description: "尝试修改搜索关键词或筛选条件。",
    icon: SearchX,
  },
  "no-selection": {
    title: "选择资源查看详情",
    description: "点击资源卡片以查看详细信息和可用操作。",
    icon: Search,
  },
  "file-missing": {
    title: "文件已缺失",
    description: "该资源文件在磁盘上已不存在。",
    icon: AlertTriangle,
  },
  "no-recent-activity": {
    title: "暂无最近活动",
    description: "打开、定位或复制资源后会显示在这里。",
    icon: CalendarX,
  },
  "no-recent-filter-results": {
    title: "当前筛选没有活动",
    description: "尝试切换时间范围或动作类型。",
    icon: SearchX,
  },
};

export function EmptyState({ variant }: Props) {
  const { title, description, icon: Icon } = content[variant];
  return (
    <div className="empty-state">
      <div className="empty-state-icon">
        <Icon size={34} aria-hidden="true" />
      </div>
      <div className="empty-state-title">{title}</div>
      <div className="empty-state-desc">{description}</div>
    </div>
  );
}
