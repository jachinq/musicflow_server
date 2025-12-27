//! 艺术家响应模型 (Subsonic API 格式)
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use crate::models::dto::{ArtistDto, ArtistDetailDto};

/// 艺术家响应 (Subsonic 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_count: Option<i32>,
}

// DTO -> Response 转换
impl From<ArtistDto> for ArtistResponse {
    fn from(dto: ArtistDto) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            cover_art: None,
            album_count: None,
        }
    }
}

impl From<ArtistDetailDto> for ArtistResponse {
    fn from(dto: ArtistDetailDto) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            cover_art: dto.cover_art_path,
            album_count: Some(dto.album_count),
        }
    }
}

impl ArtistResponse {
    pub fn from_dtos(dtos: Vec<ArtistDto>) -> Vec<Self> {
        dtos.into_iter().map(|dto| Self::from(dto)).collect()
    }

    pub fn from_detail_dtos(dtos: Vec<ArtistDetailDto>) -> Vec<Self> {
        dtos.into_iter().map(|dto| Self::from(dto)).collect()
    }
}

/// 艺术家详情 (包含专辑列表)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistDetail {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    pub album_count: i32,
    pub album: Vec<super::AlbumResponse>,
}

/// 艺术家索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indexes {
    pub last_modified: i64,
    #[serde(rename = "index")]
    pub indexes: Vec<Index>,
}

/// 按字母分组的索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub artist: Vec<ArtistResponse>,
}
