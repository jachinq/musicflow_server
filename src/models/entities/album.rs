//! 专辑数据库实体

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 专辑实体 - 对应 albums 表的完整结构
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
