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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub note: String,
    pub is_favorite: bool,
    pub is_missing: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl ScanJobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
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
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub found_count: i64,
    pub added_count: i64,
    pub updated_count: i64,
    pub unchanged_count: i64,
    pub missing_count: i64,
    pub skipped_count: i64,
    pub current_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThumbnailStatus {
    None,
    Queued,
    Generating,
    Ready,
    Failed,
}

impl ThumbnailStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Queued => "queued",
            Self::Generating => "generating",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

impl std::str::FromStr for ThumbnailStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "queued" => Ok(Self::Queued),
            "generating" => Ok(Self::Generating),
            "ready" => Ok(Self::Ready),
            "failed" => Ok(Self::Failed),
            _ => Err(anyhow::anyhow!("unknown thumbnail status: {}", s)),
        }
    }
}
