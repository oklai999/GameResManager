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
  updated_at: string;
};
