CREATE INDEX IF NOT EXISTS idx_assets_file_size ON assets(file_size);
CREATE INDEX IF NOT EXISTS idx_assets_modified_at ON assets(modified_at);
CREATE INDEX IF NOT EXISTS idx_assets_dimensions ON assets(width, height);
CREATE INDEX IF NOT EXISTS idx_assets_type_name ON assets(asset_type, file_name);
CREATE INDEX IF NOT EXISTS idx_assets_folder_name ON assets(library_folder_id, file_name);
CREATE INDEX IF NOT EXISTS idx_assets_favorite_name ON assets(is_favorite, file_name);
CREATE INDEX IF NOT EXISTS idx_assets_missing_name ON assets(is_missing, file_name);
CREATE INDEX IF NOT EXISTS idx_collection_assets_collection_asset ON collection_assets(collection_id, asset_id);
CREATE INDEX IF NOT EXISTS idx_asset_tags_tag_asset ON asset_tags(tag_id, asset_id);
