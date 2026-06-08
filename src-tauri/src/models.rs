use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Image,
    Audio,
    Video,
    Font,
    Model3d,
    Spine,
    Other,
}

impl AssetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Font => "font",
            Self::Model3d => "model3d",
            Self::Spine => "spine",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryFolder {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub last_scanned_at: Option<String>,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Asset {
    pub id: i64,
    pub library_folder_id: i64,
    pub absolute_path: String,
    pub file_name: String,
    pub extension: String,
    pub asset_type: String,
    pub file_size: i64,
    pub modified_at: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub thumbnail_path: Option<String>,
    pub thumbnail_status: String,
    pub thumbnail_error: Option<String>,
    pub note: String,
    pub is_favorite: bool,
    pub is_missing: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecentAssetAction {
    pub id: i64,
    pub asset_id: i64,
    pub action_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScanJobStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::str::FromStr for ScanJobStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(anyhow::anyhow!("unknown scan job status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanJob {
    pub id: i64,
    pub library_folder_id: i64,
    pub status: ScanJobStatus,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub found_count: i64,
    pub added_count: i64,
    pub updated_count: i64,
    pub unchanged_count: i64,
    pub missing_count: i64,
    pub skipped_count: i64,
    pub current_path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSettings {
    pub id: i64,
    pub include_images: bool,
    pub include_audio: bool,
    pub include_video: bool,
    pub include_fonts: bool,
    pub include_models: bool,
    pub include_spine: bool,
    pub include_psd: bool,
    pub generate_psd_thumbnails: bool,
    pub ignored_directory_names: String,
    pub thumbnail_cache_dir: Option<String>,
    pub database_path: Option<String>,
    pub ignored_extensions: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderAssetCounts {
    pub folder_id: i64,
    pub total: i64,
    pub missing: i64,
    pub is_accessible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPathVariants {
    pub absolute_path: String,
    pub forward_slash_path: String,
    pub folder_path: String,
    pub file_name: String,
    pub godot_res_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSearchRequest {
    pub query: String,
    pub search_file_name: bool,
    pub search_note: bool,
    pub search_path: bool,
    pub search_tags: bool,
    pub asset_type: Option<String>,
    pub library_folder_id: Option<i64>,
    pub collection_id: Option<i64>,
    pub is_favorite: Option<bool>,
    pub is_missing: Option<bool>,
    pub min_file_size: Option<i64>,
    pub max_file_size: Option<i64>,
    pub min_width: Option<i64>,
    pub max_width: Option<i64>,
    pub min_height: Option<i64>,
    pub max_height: Option<i64>,
    pub modified_after: Option<String>,
    pub modified_before: Option<String>,
    pub sort_by: String,
    pub sort_direction: String,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSearchResponse {
    pub assets: Vec<Asset>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}

