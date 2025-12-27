//! 艺术家模型

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 艺术家实体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Artist {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_brainz_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art_path: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

/// 艺术家响应（Subsonic 格式）
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ArtistResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@coverArt")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@albumCount")]
    pub album_count: Option<i32>,
}

/// 艺术家详情（包含专辑列表）
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistDetail {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@coverArt")]
    pub cover_art: Option<String>,
    #[serde(rename = "@albumCount")]
    pub album_count: i32,
    pub album: Vec<super::album::AlbumResponse>,
}

/// 艺术家索引
#[derive(Debug, Serialize, Deserialize)]
pub struct Indexes {
    #[serde(rename = "@lastModified")]
    pub last_modified: i64,
    #[serde(rename = "index")]
    pub indexes: Vec<Index>,
}

/// 按字母分组的索引
#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    #[serde(rename = "@name")]
    pub name: String,
    pub artist: Vec<ArtistResponse>,
}

impl Artist {
    pub fn new(name: String, music_brainz_id: Option<String>, cover_art_path: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            music_brainz_id,
            cover_art_path,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl From<Artist> for ArtistResponse {
    fn from(artist: Artist) -> Self {
        Self {
            id: artist.id,
            name: artist.name,
            cover_art: artist.cover_art_path,
            album_count: None, // 需要在查询时填充
        }
    }
}