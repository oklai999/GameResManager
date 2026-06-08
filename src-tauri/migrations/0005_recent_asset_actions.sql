CREATE TABLE IF NOT EXISTS recent_asset_actions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  asset_id INTEGER NOT NULL,
  action_type TEXT NOT NULL CHECK (action_type IN ('open_file', 'reveal_folder', 'copy_path')),
  created_at TEXT NOT NULL,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recent_asset_actions_created_at
ON recent_asset_actions(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_recent_asset_actions_asset_id
ON recent_asset_actions(asset_id);
