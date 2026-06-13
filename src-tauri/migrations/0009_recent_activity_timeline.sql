PRAGMA foreign_keys = OFF;

CREATE TABLE recent_asset_actions_new (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  asset_id INTEGER NOT NULL,
  action_type TEXT NOT NULL CHECK (
    action_type IN ('open_file', 'reveal_folder', 'copy_path', 'preview_media')
  ),
  created_at TEXT NOT NULL,
  FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

INSERT INTO recent_asset_actions_new (id, asset_id, action_type, created_at)
SELECT id, asset_id, action_type, created_at FROM recent_asset_actions;

DROP TABLE recent_asset_actions;
ALTER TABLE recent_asset_actions_new RENAME TO recent_asset_actions;

CREATE INDEX idx_recent_asset_actions_created_at
ON recent_asset_actions(created_at DESC, id DESC);

CREATE INDEX idx_recent_asset_actions_asset_created
ON recent_asset_actions(asset_id, created_at DESC, id DESC);

CREATE INDEX idx_recent_asset_actions_type_created
ON recent_asset_actions(action_type, created_at DESC, id DESC);

PRAGMA foreign_keys = ON;
