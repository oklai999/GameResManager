-- Deduplicate stale running jobs before creating the partial unique index.
-- For each folder, keep only the newest running job and mark older ones as failed.
UPDATE scan_jobs
SET status = 'failed',
    finished_at = datetime('now'),
    current_path = NULL,
    error_message = 'interrupted by migration: duplicate running job deduplication'
WHERE id NOT IN (
    SELECT MAX(id)
    FROM scan_jobs
    WHERE status = 'running'
    GROUP BY library_folder_id
)
  AND status = 'running';

CREATE UNIQUE INDEX IF NOT EXISTS idx_scan_jobs_folder_running ON scan_jobs(library_folder_id) WHERE status = 'running';
