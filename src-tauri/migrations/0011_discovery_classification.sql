ALTER TABLE assets ADD COLUMN parent_path TEXT NOT NULL DEFAULT '';
UPDATE assets SET parent_path = rtrim(substr(replace(absolute_path, char(92), '/'), 1, length(absolute_path) - length(file_name)), '/');
CREATE INDEX idx_assets_parent_path ON assets(parent_path COLLATE NOCASE);
CREATE TRIGGER assets_parent_insert AFTER INSERT ON assets BEGIN
  UPDATE assets SET parent_path = rtrim(substr(replace(NEW.absolute_path, char(92), '/'), 1, length(NEW.absolute_path) - length(NEW.file_name)), '/') WHERE id = NEW.id;
END;
CREATE TRIGGER assets_parent_update AFTER UPDATE OF absolute_path, file_name ON assets BEGIN
  UPDATE assets SET parent_path = rtrim(substr(replace(NEW.absolute_path, char(92), '/'), 1, length(NEW.absolute_path) - length(NEW.file_name)), '/') WHERE id = NEW.id;
END;

CREATE TABLE tag_dimensions (
  tag_id INTEGER PRIMARY KEY REFERENCES tags(id) ON DELETE CASCADE,
  dimension TEXT NOT NULL CHECK (dimension IN ('usage','subject','action','style','general'))
);
CREATE TABLE classification_rules (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  field TEXT NOT NULL CHECK (field IN ('directory','filename')),
  pattern TEXT NOT NULL,
  tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE
);
CREATE TABLE classification_batches (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  rule_name TEXT NOT NULL,
  created_at TEXT NOT NULL,
  added_count INTEGER NOT NULL DEFAULT 0,
  undone INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE classification_links (
  asset_id INTEGER NOT NULL,
  tag_id INTEGER NOT NULL,
  batch_id INTEGER NOT NULL REFERENCES classification_batches(id),
  PRIMARY KEY(asset_id, tag_id),
  FOREIGN KEY(asset_id, tag_id) REFERENCES asset_tags(asset_id, tag_id) ON DELETE CASCADE
);
CREATE TABLE classification_exclusions (
  asset_id INTEGER NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
  tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY(asset_id, tag_id)
);
-- An explicit manual add (including INSERT OR IGNORE) takes ownership.
-- Rule application only inserts absent links, then registers its ownership.
CREATE TRIGGER classification_manual_add BEFORE INSERT ON asset_tags BEGIN
  DELETE FROM classification_links WHERE asset_id = NEW.asset_id AND tag_id = NEW.tag_id;
  DELETE FROM classification_exclusions WHERE asset_id = NEW.asset_id AND tag_id = NEW.tag_id;
END;
CREATE TRIGGER classification_manual_remove BEFORE DELETE ON asset_tags BEGIN
  INSERT OR IGNORE INTO classification_exclusions(asset_id, tag_id)
  SELECT OLD.asset_id, OLD.tag_id WHERE EXISTS (SELECT 1 FROM assets WHERE id = OLD.asset_id) AND EXISTS (SELECT 1 FROM tags WHERE id = OLD.tag_id);
END;
-- Renaming a tag is an explicit semantic edit; undo must not remove it later.
CREATE TRIGGER classification_manual_rename BEFORE UPDATE OF name ON tags WHEN OLD.name != NEW.name BEGIN
  DELETE FROM classification_links WHERE tag_id = OLD.id;
END;
