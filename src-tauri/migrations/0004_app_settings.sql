ALTER TABLE scan_settings ADD COLUMN thumbnail_cache_dir TEXT;
ALTER TABLE scan_settings ADD COLUMN database_path TEXT;
ALTER TABLE scan_settings ADD COLUMN ignored_extensions TEXT NOT NULL DEFAULT '';

-- Append .import to existing ignored_directory_names if not already present
UPDATE scan_settings
SET ignored_directory_names = ignored_directory_names || ',.import'
WHERE id = 1
  AND ',' || ignored_directory_names || ',' NOT LIKE '%,.import,%';
