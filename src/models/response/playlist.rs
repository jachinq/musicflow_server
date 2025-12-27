//! 播放列表响应模型 (Subsonic API 格式)

use serde::{Deserialize, Serialize};
use crate::models::dto::PlaylistDto;
use super::SongResponse;

/// 播放列表响应 (Subsonic 格式 - 简略)
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 播放列表详情 (包含歌曲列表)
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub entry: Vec<SongResponse>,
}

/// 播放列表列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlists {
    #[serde(rename = "playlist")]
    pub playlists: Vec<PlaylistResponse>,
}

// DTO -> Response 转换
impl From<PlaylistDto> for PlaylistResponse {
    fn from(dto: PlaylistDto) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            owner: dto.owner,
            public: dto.is_public,
            song_count: dto.song_count,
            duration: Some(dto.duration),
        }
    }
}
