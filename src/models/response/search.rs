//! 搜索结果响应模型

use super::ToXml;
use serde::{Deserialize, Serialize};

/// 搜索结果响应包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultResponse {
    pub search_result: SearchResult,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(rename = "artist")]
    pub artist: Vec<super::ArtistResponse>,
    #[serde(rename = "album")]
    pub album: Vec<super::AlbumResponse>,
    #[serde(rename = "song")]
    pub song: Vec<super::Song>,
}

/// 搜索结果2响应包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult2Response {
    pub search_result2: SearchResult2,
}

/// 搜索结果2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult2 {
    #[serde(rename = "artist")]
    pub artist: Vec<super::ArtistResponse>,
    #[serde(rename = "album")]
    pub album: Vec<super::AlbumResponse>,
    #[serde(rename = "song")]
    pub song: Vec<super::Song>,
}

/// 搜索结果3响应包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult3Response {
    pub search_result3: SearchResult3,
}

/// 搜索结果3 (包含更多详情)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult3 {
    #[serde(rename = "artist")]
    pub artist: Vec<super::ArtistResponse>,
    #[serde(rename = "album")]
    pub album: Vec<super::AlbumResponse>,
    #[serde(rename = "song")]
    pub song: Vec<super::Song>,
}

// ========== XML 序列化实现 ==========

impl ToXml for SearchResultResponse {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<searchResult>");
        for artist in &self.search_result.artist {
            xml.push_str(&artist.to_xml_element());
        }
        for album in &self.search_result.album {
            xml.push_str(&album.to_xml_element());
        }
        for song in &self.search_result.song {
            xml.push_str(&song.to_xml_element());
        }
        xml.push_str("</searchResult>");
        xml
    }
}

impl ToXml for SearchResult2Response {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<searchResult2>");
        for artist in &self.search_result2.artist {
            xml.push_str(&artist.to_xml_element());
        }
        for album in &self.search_result2.album {
            xml.push_str(&album.to_xml_element());
        }
        for song in &self.search_result2.song {
            xml.push_str(&song.to_xml_element());
        }
        xml.push_str("</searchResult2>");
        xml
    }
}

impl ToXml for SearchResult3Response {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<searchResult3>");
        for artist in &self.search_result3.artist {
            xml.push_str(&artist.to_xml_element());
        }
        for album in &self.search_result3.album {
            xml.push_str(&album.to_xml_element());
        }
        for song in &self.search_result3.song {
            xml.push_str(&song.to_xml_element());
        }
        xml.push_str("</searchResult3>");
        xml
    }
}
