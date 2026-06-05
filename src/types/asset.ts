export type Asset = {
  id: number;
  library_folder_id: number;
  absolute_path: string;
  file_name: string;
  extension: string;
  asset_type: "image" | "audio" | "video" | "font" | "model3d" | "spine" | "other";
  file_size: number;
  modified_at: string;
  width: number | null;
  height: number | null;
  thumbnail_path: string | null;
  thumbnail_status: "none" | "queued" | "generating" | "ready" | "failed";
  thumbnail_error: string | null;
  note: string;
  is_favorite: boolean;
  is_missing: boolean;
  created_at: string;
  updated_at: string;
  tags?: string[];
};

export type LibraryFolder = {
  id: number;
  name: string;
  path: string;
  created_at: string;
  last_scanned_at: string | null;
  is_enabled: boolean;
};

export type SearchScope = {
  fileName: boolean;
  tag: boolean;
  note: boolean;
  path: boolean;
};

export type ScanJobStatus = "running" | "completed" | "failed" | "cancelled";

export type ScanJob = {
  id: number;
  library_folder_id: number;
  status: ScanJobStatus;
  started_at: string;
  finished_at: string | null;
  cancelled_at: string | null;
  found_count: number;
  added_count: number;
  updated_count: number;
  unchanged_count: number;
  missing_count: number;
  skipped_count: number;
  current_path: string | null;
  error_message: string | null;
};

export type AssetSearchRequest = {
  query: string;
  search_file_name: boolean;
  search_note: boolean;
  search_path: boolean;
  search_tags: boolean;
  asset_type: string | null;
  library_folder_id: number | null;
  collection_id: number | null;
  is_favorite: boolean | null;
  is_missing: boolean | null;
  limit: number;
  offset: number;
};

export type RecentAssetAction = {
  id: number;
  asset_id: number;
  action_type: "open_file" | "reveal_folder" | "copy_path";
  created_at: string;
};

export type AssetPathVariants = {
  absolute_path: string;
  forward_slash_path: string;
  folder_path: string;
  file_name: string;
  godot_res_path: string | null;
};

export type Tag = {
  id: number;
  name: string;
  color: string;
};

export type Collection = {
  id: number;
  name: string;
  description: string;
};

export type FolderAssetCounts = {
  folder_id: number;
  total: number;
  missing: number;
  is_accessible: boolean;
};

export type ScanSettings = {
  id: number;
  include_images: boolean;
  include_audio: boolean;
  include_video: boolean;
  include_fonts: boolean;
  include_models: boolean;
  include_spine: boolean;
  include_psd: boolean;
  generate_psd_thumbnails: boolean;
  ignored_directory_names: string;
  thumbnail_cache_dir: string | null;
  database_path: string | null;
  ignored_extensions: string;
  updated_at: string;
};
