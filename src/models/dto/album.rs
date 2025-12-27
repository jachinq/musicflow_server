//! 专辑数据传输对象

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 专辑基础信息 DTO (用于列表和搜索)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AlbumDto {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub year: Option<i32>,
    pub song_count: i32,
}

/// 专辑详细信息 DTO (包含更多字段)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AlbumDetailDto {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub artist_id: String,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub cover_art_path: Option<String>,
    pub song_count: i32,
    pub duration: i32,
    pub play_count: i32,
}
