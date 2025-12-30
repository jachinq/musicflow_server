//! 播放列表响应模型 (Subsonic API 格式)

use serde::{Deserialize, Serialize};
use crate::models::dto::PlaylistDto;
use super::{Song, ToXml};

/// 播放列表响应 (Subsonic 格式 - 简略)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistResponse {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub public: bool,
    pub song_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
}

/// 播放列表详情 (包含歌曲列表)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistDetail {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub public: bool,
    pub song_count: i32,
    pub duration: i32,
    pub entry: Vec<Song>,
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

// ========== XML 序列化实现 ==========

impl ToXml for PlaylistResponse {
    fn to_xml_element(&self) -> String {
        let mut xml = format!(
            r#"<playlist id="{}" name="{}" owner="{}" public="{}" songCount="{}""#,
            self.id, self.name, self.owner, self.public, self.song_count
        );
        if let Some(duration) = self.duration {
            xml.push_str(&format!(r#" duration="{}""#, duration));
        }
        xml.push_str("/>");
        xml
    }
}

impl ToXml for Playlists {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<playlists>");
        for playlist in &self.playlists {
            xml.push_str(&playlist.to_xml_element());
        }
        xml.push_str("</playlists>");
        xml
    }
}

impl ToXml for PlaylistDetail {
    fn to_xml_element(&self) -> String {
        let mut xml = format!(
            r#"<playlist id="{}" name="{}" owner="{}" public="{}" songCount="{}" duration="{}">"#,
            self.id, self.name, self.owner, self.public, self.song_count, self.duration
        );
        for song in &self.entry {
            xml.push_str(&song.to_xml_element());
        }
        xml.push_str("</playlist>");
        xml
    }
}
