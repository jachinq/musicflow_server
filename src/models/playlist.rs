//! 播放列表模型

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 播放列表实体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Playlist {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub comment: Option<String>,
    pub is_public: bool,
    pub song_count: i32,
    pub duration: i32,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

/// 播放列表响应（简略）
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@owner")]
    pub owner: String,
    #[serde(rename = "@public")]
    pub public: bool,
    #[serde(rename = "@songCount")]
    pub song_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@duration")]
    pub duration: Option<i32>,
}

/// 播放列表详情（包含歌曲）
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistDetail {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@owner")]
    pub owner: String,
    #[serde(rename = "@public")]
    pub public: bool,
    #[serde(rename = "@songCount")]
    pub song_count: i32,
    #[serde(rename = "@duration")]
    pub duration: i32,
    pub entry: Vec<super::song::SongResponse>,
}

/// 播放列表列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct Playlists {
    #[serde(rename = "playlist")]
    pub playlists: Vec<PlaylistResponse>,
}

/// 创建播放列表请求
#[derive(Debug, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub song_id: Option<Vec<String>>,
}

/// 更新播放列表请求
#[derive(Debug, Deserialize)]
pub struct UpdatePlaylistRequest {
    pub name: Option<String>,
    pub comment: Option<String>,
    pub public: Option<bool>,
    pub song_id_to_add: Option<Vec<String>>,
    pub song_index_to_remove: Option<Vec<i32>>,
}

impl Playlist {
    pub fn new(owner_id: String, name: String, comment: Option<String>, is_public: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            owner_id,
            name,
            comment,
            is_public,
            song_count: 0,
            duration: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}