//! 专辑模型

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 专辑实体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Album {
    pub id: String,
    pub artist_id: String,
    pub name: String,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub cover_art_path: Option<String>,
    pub path: String,
    pub song_count: i32,
    pub duration: i32,
    pub play_count: i32,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

/// 专辑响应（Subsonic 格式 - 简略）
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AlbumResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@artist")]
    pub artist: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@artistId")]
    pub artist_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@coverArt")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@songCount")]
    pub song_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@created")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@duration")]
    pub duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@playCount")]
    pub play_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@year")]
    pub year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@genre")]
    pub genre: Option<String>,
}

/// 专辑详情（包含歌曲列表）
#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumDetail {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@artist")]
    pub artist: String,
    #[serde(rename = "@artistId")]
    pub artist_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@coverArt")]
    pub cover_art: Option<String>,
    #[serde(rename = "@songCount")]
    pub song_count: i32,
    #[serde(rename = "@duration")]
    pub duration: i32,
    pub song: Vec<super::song::SongResponse>,
}

/// 专辑列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumList {
    #[serde(rename = "album")]
    pub albums: Vec<AlbumResponse>,
}

/// 专辑列表2（包含更多详情）
#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumList2 {
    #[serde(rename = "album")]
    pub albums: Vec<AlbumResponse>,
}

impl Album {
    pub fn new(
        artist_id: String,
        name: String,
        path: String,
        year: Option<i32>,
        genre: Option<String>,
        cover_art_path: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            artist_id,
            name,
            year,
            genre,
            cover_art_path,
            path,
            song_count: 0,
            duration: 0,
            play_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}