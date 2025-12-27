//! 收藏模型
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 收藏实体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Starred {
    pub id: String,
    pub user_id: String,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub song_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

/// 收藏响应（Subsonic 格式）
#[derive(Debug, Serialize, Deserialize)]
pub struct StarredResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<Vec<super::artist::ArtistResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<Vec<super::album::AlbumResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub song: Option<Vec<super::song::SongResponse>>,
}

/// 收藏请求
#[derive(Debug, Deserialize)]
pub struct StarRequest {
    pub id: Option<String>,
    pub album_id: Option<String>,
    pub artist_id: Option<String>,
}

impl Starred {
    pub fn new(user_id: String, artist_id: Option<String>, album_id: Option<String>, song_id: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            artist_id,
            album_id,
            song_id,
            created_at: Utc::now(),
        }
    }
}