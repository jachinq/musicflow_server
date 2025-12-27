//! 歌曲模型
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 歌曲实体
#[derive(Debug, Serialize, Deserialize, FromRow)]
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
    pub play_count: i32,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

/// 歌曲响应（Subsonic 格式）
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SongResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@artist")]
    pub artist: String,
    #[serde(rename = "@album")]
    pub album: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@genre")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@year")]
    pub year: Option<i32>,
    #[serde(rename = "@duration")]
    pub duration: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@bitRate")]
    pub bit_rate: Option<i32>,
    #[serde(rename = "@contentType")]
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@path")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@trackNumber")]
    pub track_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@discNumber")]
    pub disc_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@coverArt")]
    pub cover_art: Option<String>,
}

/// 随机歌曲响应
#[derive(Debug, Serialize, Deserialize)]
pub struct RandomSongs {
    #[serde(rename = "song")]
    pub songs: Vec<SongResponse>,
}

impl Song {
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
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
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
            play_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}