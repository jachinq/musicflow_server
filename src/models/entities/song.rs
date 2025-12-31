//! 歌曲数据库实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::utils::id_builder;

/// 歌曲实体 - 对应 songs 表的完整结构
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Song {
    pub id: String,
    pub album_id: String,
    pub artist_id: String,
    pub title: String,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub duration: i32,
    pub bit_rate: Option<i32>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub content_type: Option<String>,
    pub file_path: String,
    pub file_size: Option<i64>,
    pub lyrics: Option<String>,
    pub play_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Song {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        album_id: String,
        artist_id: String,
        title: String,
        duration: i32,
        file_path: String,
        track_number: Option<i32>,
        disc_number: Option<i32>,
        bit_rate: Option<i32>,
        genre: Option<String>,
        year: Option<i32>,
        content_type: Option<String>,
        file_size: Option<i64>,
        lyrics: Option<String>,
    ) -> Self {
        Self {
            id: id_builder::generate_id(),
            album_id,
            artist_id,
            title,
            track_number,
            disc_number,
            duration,
            bit_rate,
            genre,
            year,
            content_type,
            file_path,
            file_size,
            lyrics,
            play_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
