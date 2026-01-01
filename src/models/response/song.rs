//! 歌曲响应模型 (Subsonic API 格式)
#![allow(dead_code)]

use super::ToXml;
use crate::models::dto::{ComplexSongDto, SongDetailDto, SongDto};
use serde::{Deserialize, Serialize};

/// 歌曲响应 (Subsonic 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_starred: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
}

// DTO -> Response 转换
impl From<SongDto> for Song {
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
            cover_art: dto.cover_art,
            album_id: None,
            artist_id: None,
            size: None,
            play_count: None,
            user_rating: None,
            is_starred: None,
            suffix: None,
        }
    }
}

impl From<SongDetailDto> for Song {
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
            path: dto.path,
            track_number: dto.track_number,
            disc_number: dto.disc_number,
            cover_art: dto.cover_art,
            album_id: Some(dto.album_id),
            artist_id: Some(dto.artist_id),
            size: None,
            play_count: None,
            user_rating: None,
            is_starred: None,
            suffix: None,
        }
    }
}

impl From<ComplexSongDto> for Song {
    fn from(dto: ComplexSongDto) -> Self {
        Self {
            id: dto.song.id,
            title: dto.song.title,
            artist: dto.song.artist,
            album: dto.song.album,
            genre: dto.song.genre,
            year: dto.song.year,
            duration: dto.song.duration,
            bit_rate: dto.song.bit_rate,
            content_type: dto
                .song
                .content_type
                .unwrap_or_else(|| "audio/mpeg".to_string()),
            path: dto.song.path,
            track_number: dto.song.track_number,
            disc_number: dto.song.disc_number,
            cover_art: dto.song.cover_art,
            album_id: Some(dto.song.album_id),
            artist_id: Some(dto.song.artist_id),
            size: dto.song.file_size,
            play_count: dto.song.play_count,
            user_rating: dto.user_rating,
            is_starred: dto.is_starred,
            suffix: dto.suffix,
        }
    }
}

// 批量 DTO -> Response 转换
impl Song {
    pub fn from_dtos(dtos: Vec<SongDto>) -> Vec<Self> {
        dtos.into_iter().map(|dto| dto.into()).collect()
    }
    pub fn from_detail_dtos(dtos: Vec<SongDetailDto>) -> Vec<Self> {
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
    pub song: Vec<Song>,
}

/// 歌曲列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Songs {
    #[serde(rename = "song")]
    pub songs: Vec<Song>,
}

/// 热门歌曲响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopSongsResponse {
    pub top_songs: TopSongs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopSongs {
    pub song: Vec<Song>,
}

/// 流派歌曲响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongsByGenreResponse {
    pub songs_by_genre: SongsResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongsResponse {
    pub song: Vec<Song>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongResponse {
    pub song: Song,
}

// ========== XML 序列化实现 ==========

impl ToXml for Song {
    fn to_xml_element(&self) -> String {
        let mut xml = format!(
            r#"<song id="{}" title="{}" artist="{}" album="{}" duration="{}" contentType="{}""#,
            self.id, self.title, self.artist, self.album, self.duration, self.content_type
        );
        if let Some(genre) = &self.genre {
            xml.push_str(&format!(r#" genre="{}""#, genre));
        }
        if let Some(year) = self.year {
            xml.push_str(&format!(r#" year="{}""#, year));
        }
        if let Some(bit_rate) = self.bit_rate {
            xml.push_str(&format!(r#" bitRate="{}""#, bit_rate));
        }
        if let Some(path) = &self.path {
            xml.push_str(&format!(r#" path="{}""#, path));
        }
        if let Some(track_number) = self.track_number {
            xml.push_str(&format!(r#" track="{}""#, track_number));
        }
        if let Some(disc_number) = self.disc_number {
            xml.push_str(&format!(r#" discNumber="{}""#, disc_number));
        }
        if let Some(cover_art) = &self.cover_art {
            xml.push_str(&format!(r#" coverArt="{}""#, cover_art));
        }
        if let Some(album_id) = &self.album_id {
            xml.push_str(&format!(r#" albumId="{}""#, album_id));
        }
        if let Some(artist_id) = &self.artist_id {
            xml.push_str(&format!(r#" artistId="{}""#, artist_id));
        }
        if let Some(value) = &self.size {
            xml.push_str(&format!(r#" size="{}""#, value));
        }
        if let Some(value) = &self.suffix {
            xml.push_str(&format!(r#" suffix="{}""#, value));
        }
        if let Some(value) = &self.user_rating {
            xml.push_str(&format!(r#" userRating="{}""#, value));
        }
        if let Some(value) = &self.is_starred {
            xml.push_str(&format!(r#" isStarred="{}""#, value));
        }
        if let Some(value) = &self.suffix {
            xml.push_str(&format!(r#" suffix="{}""#, value));
        }
        xml.push_str("/>");
        xml
    }
}

impl ToXml for RandomSongsResponse {
    fn to_xml_element(&self) -> String {
        self.random_songs.to_xml_element()
    }
}

impl ToXml for RandomSongs {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<randomSongs>");
        for song in &self.song {
            xml.push_str(&song.to_xml_element());
        }
        xml.push_str("</randomSongs>");
        xml
    }
}

impl ToXml for TopSongsResponse {
    fn to_xml_element(&self) -> String {
        self.top_songs.to_xml_element()
    }
}

impl ToXml for TopSongs {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<topSongs>");
        for song in &self.song {
            xml.push_str(&song.to_xml_element());
        }
        xml.push_str("</topSongs>");
        xml
    }
}

impl ToXml for SongsByGenreResponse {
    fn to_xml_element(&self) -> String {
        self.songs_by_genre.to_xml_element()
    }
}

impl ToXml for SongsResponse {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<songsByGenre>");
        for song in &self.song {
            xml.push_str(&song.to_xml_element());
        }
        xml.push_str("</songsByGenre>");
        xml
    }
}

impl ToXml for SongResponse {
    fn to_xml_element(&self) -> String {
        self.song.to_xml_element()
    }
}
