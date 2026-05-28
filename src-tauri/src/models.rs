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
