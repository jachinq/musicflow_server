//! 艺术家响应模型 (Subsonic API 格式)
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use crate::models::dto::{ArtistDetailDto, ArtistDto};
use crate::models::entities::Artist;
use super::ToXml;

/// 艺术家响应 (Subsonic 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistsResponse {
    pub artists: Artists,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artists {
    pub index: Vec<ArtistIndex>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistIndex {
    pub name: String,
    pub artist: Vec<ArtistResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_count: Option<i32>,
}


// Entity -> Response 转换
impl From<Artist> for ArtistResponse {
    fn from(dto: Artist) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            cover_art: dto.cover_art_path,
            album_count: None,
        }
    }
}

// DTO -> Response 转换
impl From<ArtistDto> for ArtistResponse {
    fn from(dto: ArtistDto) -> Self {
        Self {
            id: dto.id.to_string(),
            name: dto.name,
            cover_art: Some(format!("ar-{}", dto.id)),
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
            album_count: None,
        }
    }
}

impl ArtistResponse {
    pub fn from_entities(entities: Vec<Artist>) -> Vec<Self> {
        entities.into_iter().map(Self::from).collect()
    }
    pub fn from_dtos(dtos: Vec<ArtistDto>) -> Vec<Self> {
        dtos.into_iter().map(Self::from).collect()
    }

    pub fn from_detail_dtos(dtos: Vec<ArtistDetailDto>) -> Vec<Self> {
        dtos.into_iter().map(Self::from).collect()
    }
}

/// 艺术家详情 (包含专辑列表)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistDetailResponse {
    pub artist: ArtistDetail,
}
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

// ========== XML 序列化实现 ==========

impl ToXml for ArtistResponse {
    fn to_xml_element(&self) -> String {
        let mut xml = format!(r#"<artist id="{}" name="{}""#, self.id, self.name);
        if let Some(cover_art) = &self.cover_art {
            xml.push_str(&format!(r#" coverArt="{}""#, cover_art));
        }
        if let Some(album_count) = self.album_count {
            xml.push_str(&format!(r#" albumCount="{}""#, album_count));
        }
        xml.push_str("/>");
        xml
    }
}

impl ToXml for Indexes {
    fn to_xml_element(&self) -> String {
        let mut xml = format!(r#"<indexes lastModified="{}">"#, self.last_modified);
        for index in &self.indexes {
            xml.push_str(&index.to_xml_element());
        }
        xml.push_str("</indexes>");
        xml
    }
}

impl ToXml for Index {
    fn to_xml_element(&self) -> String {
        let mut xml = format!(r#"<index name="{}">"#, self.name);
        for artist in &self.artist {
            xml.push_str(&artist.to_xml_element());
        }
        xml.push_str("</index>");
        xml
    }
}

impl ToXml for ArtistsResponse {
    fn to_xml_element(&self) -> String {
        self.artists.to_xml_element()
    }
}

impl ToXml for Artists {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<artists>");
        for index in &self.index {
            xml.push_str(&index.to_xml_element());
        }
        xml.push_str("</artists>");
        xml
    }
}

impl ToXml for ArtistIndex {
    fn to_xml_element(&self) -> String {
        let mut xml = format!(r#"<index name="{}">"#, self.name);
        for artist in &self.artist {
            xml.push_str(&artist.to_xml_element());
        }
        xml.push_str("</index>");
        xml
    }
}

impl ToXml for ArtistDetailResponse {
    fn to_xml_element(&self) -> String {
        self.artist.to_xml_element()
    }
}

impl ToXml for ArtistDetail {
    fn to_xml_element(&self) -> String {
        let mut xml = format!(
            r#"<artist id="{}" name="{}" albumCount="{}""#,
            self.id, self.name, self.album_count
        );
        if let Some(cover_art) = &self.cover_art {
            xml.push_str(&format!(r#" coverArt="{}""#, cover_art));
        }
        xml.push('>');
        for album in &self.album {
            xml.push_str(&album.to_xml_element());
        }
        xml.push_str("</artist>");
        xml
    }
}
