//! 歌曲响应模型 (Subsonic API 格式)
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use crate::models::dto::{SongDto, SongDetailDto};

/// 歌曲响应 (Subsonic 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongResponse {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    pub duration: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<i32>,
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disc_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
}

// DTO -> Response 转换
impl From<SongDto> for SongResponse {
    fn from(dto: SongDto) -> Self {
        Self {
            id: dto.id,
            title: dto.title,
            artist: dto.artist,
            album: dto.album,
            genre: None,
            year: None,
            duration: dto.duration,
            bit_rate: None,
            content_type: dto.content_type.unwrap_or_else(|| "audio/mpeg".to_string()),
            path: None,
            track_number: None,
            disc_number: None,
            cover_art: None,
        }
    }
}

impl From<SongDetailDto> for SongResponse {
    fn from(dto: SongDetailDto) -> Self {
        Self {
            id: dto.id,
            title: dto.title,
            artist: dto.artist,
            album: dto.album,
            genre: dto.genre,
            year: dto.year,
            duration: dto.duration,
            bit_rate: dto.bit_rate,
            content_type: dto.content_type.unwrap_or_else(|| "audio/mpeg".to_string()),
            path: None,
            track_number: dto.track_number,
            disc_number: dto.disc_number,
            cover_art: dto.cover_art_path,
        }
    }
}

// 批量 DTO -> Response 转换
impl SongResponse {
    pub fn from_dtos(dtos: Vec<SongDto>) -> Vec<Self> {
        dtos.into_iter().map(|dto| dto.into()).collect()
    }
}

/// 随机歌曲响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RandomSongsResponse {
    pub random_songs: RandomSongs,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomSongs {
    pub song: Vec<SongResponse>,
}

/// 歌曲列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Songs {
    #[serde(rename = "song")]
    pub songs: Vec<SongResponse>,
}

/// 热门歌曲响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopSongsResponse {
    pub top_songs: TopSongs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopSongs {
    pub song: Vec<SongResponse>,
}
