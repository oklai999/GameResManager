ALTER TABLE scan_jobs ADD COLUMN cancelled_at TEXT;
ALTER TABLE scan_jobs ADD COLUMN unchanged_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE scan_jobs ADD COLUMN current_path TEXT;
ALTER TABLE assets ADD COLUMN thumbnail_status TEXT NOT NULL DEFAULT 'none';
ALTER TABLE assets ADD COLUMN thumbnail_error TEXT;
CREATE TABLE IF NOT EXISTS scan_seen_paths (
  scan_job_id INTEGER NOT NULL,
  absolute_path TEXT NOT NULL,
  PRIMARY KEY (scan_job_id, absolute_path),
  FOREIGN KEY (scan_job_id) REFERENCES scan_jobs(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS scan_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  include_images INTEGER NOT NULL DEFAULT 1,
  include_audio INTEGER NOT NULL DEFAULT 1,
  include_video INTEGER NOT NULL DEFAULT 1,
  include_fonts INTEGER NOT NULL DEFAULT 1,
  include_models INTEGER NOT NULL DEFAULT 1,
  include_spine INTEGER NOT NULL DEFAULT 1,
  include_psd INTEGER NOT NULL DEFAULT 1,
  generate_psd_thumbnails INTEGER NOT NULL DEFAULT 0,
  ignored_directory_names TEXT NOT NULL DEFAULT 'node_modules,.git,.godot,target,dist,build,.codex_spreadsheet_tinyswords',
  updated_at TEXT NOT NULL
);
INSERT OR IGNORE INTO scan_settings (id, updated_at) VALUES (1, datetime('now'));
CREATE INDEX IF NOT EXISTS idx_scan_seen_paths_job ON scan_seen_paths(scan_job_id);
CREATE INDEX IF NOT EXISTS idx_scan_jobs_folder_status ON scan_jobs(library_folder_id, status);
CREATE INDEX IF NOT EXISTS idx_assets_thumbnail_status ON assets(thumbnail_status);
