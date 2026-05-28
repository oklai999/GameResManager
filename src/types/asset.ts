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

export type ScanResult = {
  found: number;
  added: number;
  updated: number;
  skipped: number;
  missing: number;
};
