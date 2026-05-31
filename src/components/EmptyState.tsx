type Props = {
  variant: "no-folders" | "no-assets" | "no-results" | "file-missing";
};

const content: Record<Props["variant"], { title: string; description: string }> = {
  "no-folders": {
    title: "还没有添加素材文件夹",
    description: '点击左侧"添加文件夹"按钮，选择一个本地文件夹开始使用。',
  },
  "no-assets": {
    title: "还没有扫描资源",
    description: "选中一个素材文件夹，点击右侧的扫描按钮开始索引资源。",
  },
  "no-results": {
    title: "没有找到匹配的资源",
    description: "尝试修改搜索关键词或筛选条件。",
  },
  "file-missing": {
    title: "文件已缺失",
    description: "该资源文件在磁盘上已不存在。",
  },
};

export function EmptyState({ variant }: Props) {
  const { title, description } = content[variant];
  return (
    <div className="empty-state">
      <div className="empty-state-icon">
        {variant === "no-folders" && "📁"}
        {variant === "no-assets" && "🔍"}
        {variant === "no-results" && "🔎"}
        {variant === "file-missing" && "⚠️"}
      </div>
      <div className="empty-state-title">{title}</div>
      <div className="empty-state-desc">{description}</div>
    </div>
  );
}
