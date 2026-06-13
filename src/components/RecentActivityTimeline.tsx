import { useMemo, useState } from "react";
import { AlertTriangle, ChevronDown, ChevronUp, Heart } from "lucide-react";
import type { Asset, RecentActionType, RecentActivityItem } from "../types/asset";

const ACTION_LABELS: Record<RecentActionType, string> = {
  open_file: "打开文件",
  reveal_folder: "打开所在目录",
  copy_path: "复制路径",
  preview_media: "媒体预览",
};

type Props = {
  items: RecentActivityItem[];
  selectedIds: number[];
  totalCount: number;
  loading: boolean;
  loadingMore: boolean;
  canLoadMore: boolean;
  onSelectAsset: (assetId: number) => void;
  onOpenAsset: (asset: Asset) => void;
  onLoadMore: () => void;
};

function formatRelativeTime(iso: string): string {
  try {
    const then = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - then.getTime();
    const diffSec = Math.floor(diffMs / 1000);
    if (diffSec < 60) return "刚刚";
    const diffMin = Math.floor(diffSec / 60);
    if (diffMin < 60) return `${diffMin} 分钟前`;
    const diffHour = Math.floor(diffMin / 60);
    if (diffHour < 24) return `${diffHour} 小时前`;
    const diffDay = Math.floor(diffHour / 24);
    if (diffDay < 7) return `${diffDay} 天前`;
    return then.toLocaleDateString("zh-CN");
  } catch {
    return iso;
  }
}

function formatActionTime(iso: string): string {
  try {
    return new Date(iso).toLocaleString("zh-CN");
  } catch {
    return iso;
  }
}

function folderPath(absolutePath: string, fileName: string): string {
  if (absolutePath.endsWith(fileName)) {
    return absolutePath.slice(0, -fileName.length).replace(/\\/g, "/").replace(/\/$/, "") || "/";
  }
  const lastSlash = absolutePath.replace(/\\/g, "/").lastIndexOf("/");
  return lastSlash > 0 ? absolutePath.slice(0, lastSlash).replace(/\\/g, "/") : absolutePath.replace(/\\/g, "/");
}

function buildSummary(item: RecentActivityItem): string {
  const parts: string[] = [];
  if (item.open_file_count > 0) parts.push(`打开文件 ${item.open_file_count}`);
  if (item.reveal_folder_count > 0) parts.push(`打开所在目录 ${item.reveal_folder_count}`);
  if (item.copy_path_count > 0) parts.push(`复制路径 ${item.copy_path_count}`);
  if (item.preview_media_count > 0) parts.push(`媒体预览 ${item.preview_media_count}`);
  return parts.join(" · ") || ACTION_LABELS[item.latest_action_type];
}

export function RecentActivityTimeline({
  items,
  selectedIds,
  totalCount,
  loading,
  loadingMore,
  canLoadMore,
  onSelectAsset,
  onOpenAsset,
  onLoadMore,
}: Props) {
  const [expandedIds, setExpandedIds] = useState<Set<number>>(new Set());

  const selectedSet = useMemo(() => new Set(selectedIds), [selectedIds]);

  function toggleExpanded(assetId: number) {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(assetId)) {
        next.delete(assetId);
      } else {
        next.add(assetId);
      }
      return next;
    });
  }

  if (items.length === 0 && !loading) {
    return null;
  }

  return (
    <section className="recent-activity-timeline" aria-label="最近活动时间线">
      <div className="recent-activity-list" role="list">
        {items.map((item) => {
          const isSelected = selectedSet.has(item.asset.id);
          const isExpanded = expandedIds.has(item.asset.id);
          const isMissing = item.asset.is_missing;
          return (
            <article
              key={item.asset.id}
              className={`recent-activity-card${isSelected ? " selected" : ""}${isMissing ? " missing" : ""}`}
              role="listitem"
            >
              <button
                type="button"
                className="recent-activity-card-main"
                onClick={() => onSelectAsset(item.asset.id)}
                onDoubleClick={() => {
                  if (!isMissing) {
                    onOpenAsset(item.asset);
                  }
                }}
                aria-label={`选择 ${item.asset.file_name}`}
              >
                <div className="recent-activity-card-header">
                  <span className="recent-activity-file-name">{item.asset.file_name}</span>
                  {item.asset.is_favorite && (
                    <span className="recent-activity-favorite" aria-label="已收藏">
                      <Heart size={12} aria-hidden="true" />
                    </span>
                  )}
                  {isMissing && (
                    <span className="recent-activity-missing-badge" aria-label="文件缺失">
                      <AlertTriangle size={12} aria-hidden="true" />
                      缺失
                    </span>
                  )}
                </div>
                <div className="recent-activity-path">{folderPath(item.asset.absolute_path, item.asset.file_name)}</div>
                <div className="recent-activity-meta">
                  <span>{ACTION_LABELS[item.latest_action_type]}</span>
                  <span aria-label={`最后活动于 ${formatActionTime(item.latest_action_at)}`}>
                    {formatRelativeTime(item.latest_action_at)}
                  </span>
                </div>
                <div className="recent-activity-summary">{buildSummary(item)} · 共 {item.action_count} 次</div>
              </button>
              <div className="recent-activity-card-actions">
                <button
                  type="button"
                  className="recent-activity-expand-btn"
                  onClick={() => toggleExpanded(item.asset.id)}
                  aria-expanded={isExpanded}
                  aria-label={isExpanded ? `收起 ${item.asset.file_name} 的活动明细` : `展开 ${item.asset.file_name} 的活动明细`}
                >
                  {isExpanded ? <ChevronUp size={14} aria-hidden="true" /> : <ChevronDown size={14} aria-hidden="true" />}
                  <span>{isExpanded ? "收起" : "展开"}</span>
                </button>
              </div>
              {isExpanded && (
                <div className="recent-activity-details">
                  <ul className="recent-activity-detail-list" aria-label={`${item.asset.file_name} 的活动明细`}>
                    {item.actions.map((action) => (
                      <li key={action.id} className="recent-activity-detail-item">
                        <span>{ACTION_LABELS[action.action_type]}</span>
                        <span className="recent-activity-detail-time">{formatActionTime(action.created_at)}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </article>
          );
        })}
      </div>
      {canLoadMore && (
        <div className="recent-activity-load-more">
          <button
            type="button"
            onClick={onLoadMore}
            disabled={loadingMore}
            aria-label="加载更多活动"
          >
            {loadingMore ? "加载中..." : "加载更多"}
          </button>
        </div>
      )}
      {loading && items.length === 0 && (
        <div className="recent-activity-loading">加载中...</div>
      )}
    </section>
  );
}
