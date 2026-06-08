import type { ScanJob } from "../types/asset";

type Props = {
  job: ScanJob;
};

const statusLabels: Record<string, string> = {
  running: "扫描中",
  completed: "扫描完成",
  failed: "扫描失败",
  cancelled: "已取消",
};

export function ScanStatusBar({ job }: Props) {
  return (
    <div className={`scan-status ${job.status}`}>
      <div className="scan-status-header">
        <span className="scan-status-label">{statusLabels[job.status]}</span>
        {job.current_path && (
          <span className="scan-current-path" title={job.current_path}>
            {job.current_path}
          </span>
        )}
      </div>
      <div className="scan-status-stats">
        <span>发现 {job.found_count}</span>
        <span>新增 {job.added_count}</span>
        <span>更新 {job.updated_count}</span>
        <span>未变化 {job.unchanged_count}</span>
        <span>缺失 {job.missing_count}</span>
        <span>跳过 {job.skipped_count}</span>
      </div>
      {job.error_message && <div className="scan-error">{job.error_message}</div>}
    </div>
  );
}
