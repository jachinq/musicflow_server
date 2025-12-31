//! 收藏响应模型 (Subsonic API 格式)

use super::{AlbumResponse, ArtistResponse, Song, ToXml};
use serde::{Deserialize, Serialize};

/// 收藏响应 (Subsonic 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarredResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<Vec<ArtistResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<Vec<AlbumResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub song: Option<Vec<Song>>,
}

// ========== XML 序列化实现 ==========

impl ToXml for StarredResponse {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<starred>");

        // 添加艺术家列表
        if let Some(artists) = &self.artist {
            for artist in artists {
                xml.push_str(&artist.to_xml_element());
            }
        }

        // 添加专辑列表
        if let Some(albums) = &self.album {
            for album in albums {
                xml.push_str(&album.to_xml_element());
            }
        }

        // 添加歌曲列表
        if let Some(songs) = &self.song {
            for song in songs {
                xml.push_str(&song.to_xml_element());
            }
        }

        xml.push_str("</starred>");
        xml
    }
}
