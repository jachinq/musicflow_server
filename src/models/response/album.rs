//! 专辑响应模型 (Subsonic API 格式)
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use crate::models::dto::{AlbumDto, AlbumDetailDto};

/// 专辑响应 (Subsonic 格式 - 简略)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumResponse {
    pub id: String,
    pub name: String,
    pub artist: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub song_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
}

// DTO -> Response 转换
impl From<AlbumDto> for AlbumResponse {
    fn from(dto: AlbumDto) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            artist: dto.artist,
            artist_id: None,
            cover_art: None,
            song_count: Some(dto.song_count),
            created: None,
            duration: None,
            play_count: None,
            year: dto.year,
            genre: None,
        }
    }
}

impl From<AlbumDetailDto> for AlbumResponse {
    fn from(dto: AlbumDetailDto) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            artist: dto.artist,
            artist_id: Some(dto.artist_id),
            cover_art: dto.cover_art_path,
            song_count: Some(dto.song_count),
            created: None,
            duration: Some(dto.duration),
            play_count: Some(dto.play_count),
            year: dto.year,
            genre: dto.genre,
        }
    }
}

impl AlbumResponse {
    pub fn from_dtos(dtos: Vec<AlbumDto>) -> Vec<Self> {
        dtos.into_iter().map(|dto| AlbumResponse::from(dto)).collect()
    }

    pub fn from_dto_details(dtos: Vec<AlbumDetailDto>) -> Vec<Self> {
        dtos.into_iter().map(|dto| AlbumResponse::from(dto)).collect()
    }    
}

/// 专辑详情 (包含歌曲列表)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumDetailResponse {
    pub album: AlbumDetail,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumDetail {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub artist_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    pub song_count: i32,
    pub duration: i32,
    pub song: Vec<super::SongResponse>,
}

/// 专辑列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumList {
    #[serde(rename = "album")]
    pub albums: Vec<AlbumResponse>,
}

/// 专辑列表2 (包含更多详情)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumList2Response {
    pub album_list2: AlbumList2,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumList2 {
    #[serde(rename = "album")]
    pub albums: Vec<AlbumResponse>,
}
