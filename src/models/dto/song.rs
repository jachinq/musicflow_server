//! 歌曲数据传输对象

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 歌曲基础信息 DTO (用于列表和搜索)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SongDto {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: i32,
    pub content_type: Option<String>,
    pub cover_art: Option<String>,
}

/// 歌曲详细信息 DTO (包含所有数据库字段)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SongDetailDto {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    pub album: String,
    pub album_id: String,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub duration: i32,
    pub bit_rate: Option<i32>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub content_type: Option<String>,
    pub path: Option<String>,
    pub cover_art: Option<String>,
    pub file_size: Option<u32>,
    pub play_count: Option<i32>,
}

/// 歌曲详细信息 DTO (包含所有需要返回的字段)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ComplexSongDto {
    pub song: SongDetailDto,
    pub user_rating: Option<i32>,
    pub is_starred: Option<bool>,
    pub suffix: Option<String>,
}
