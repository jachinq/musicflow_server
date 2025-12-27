//! Subsonic API 响应模型

use serde::{Deserialize, Serialize};

/// Subsonic 响应容器
#[derive(Debug, Serialize, Deserialize)]
pub struct SubsonicResponse<T> {
    #[serde(rename = "subsonic-response")]
    pub response: ResponseContainer<T>,
}

/// 响应容器
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseContainer<T> {
    pub status: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SubsonicError>,
    #[serde(flatten)]
    pub data: Option<T>,
}

/// Subsonic 错误
#[derive(Debug, Serialize, Deserialize)]
pub struct SubsonicError {
    pub code: i32,
    pub message: String,
}

/// 系统信息
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub server: String,
    pub r#type: String,
}

/// 许可证信息
#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    pub valid: bool,
    pub email: String,
    pub key: String,
}

/// 扫描状态
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanStatus {
    pub scanning: bool,
    pub count: i32,
}

/// 艺术家索引
#[derive(Debug, Serialize, Deserialize)]
pub struct Indexes {
    #[serde(rename = "@lastModified")]
    pub last_modified: i64,
    #[serde(rename = "index")]
    pub indexes: Vec<Index>,
}

/// 索引
#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    #[serde(rename = "@name")]
    pub name: String,
    pub artist: Vec<Artist>,
}

/// 艺术家（简要）
#[derive(Debug, Serialize, Deserialize)]
pub struct Artist {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@coverArt")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@albumCount")]
    pub album_count: Option<i32>,
}

/// 艺术家详情
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistDetail {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@coverArt")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@albumCount")]
    pub album_count: Option<i32>,
    #[serde(rename = "album")]
    pub album: Option<Vec<Album>>,
}

/// 专辑（用于艺术家详情）
#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@artist")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@year")]
    pub year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@coverArt")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@songCount")]
    pub song_count: Option<i32>,
}

/// 专辑详情
#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumDetail {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@artist")]
    pub artist: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@year")]
    pub year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@coverArt")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@songCount")]
    pub song_count: Option<i32>,
    #[serde(rename = "song")]
    pub song: Option<Vec<Song>>,
}

/// 歌曲（用于API响应）
#[derive(Debug, Serialize, Deserialize)]
pub struct Song {
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
}

/// 目录
#[derive(Debug, Serialize, Deserialize)]
pub struct Directory {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@parent")]
    pub parent: Option<String>,
    pub child: Vec<Child>,
}

/// 目录子项
#[derive(Debug, Serialize, Deserialize)]
pub struct Child {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@artist")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@album")]
    pub album: Option<String>,
    #[serde(rename = "@isDir")]
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@coverArt")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@duration")]
    pub duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "@playCount")]
    pub play_count: Option<i32>,
}

/// 搜索结果
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(rename = "artist")]
    pub artist: Vec<super::artist::ArtistResponse>,
    #[serde(rename = "album")]
    pub album: Vec<super::album::AlbumResponse>,
    #[serde(rename = "song")]
    pub song: Vec<super::song::SongResponse>,
}

/// 搜索结果2
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult2 {
    #[serde(rename = "artist")]
    pub artist: Vec<super::artist::ArtistResponse>,
    #[serde(rename = "album")]
    pub album: Vec<super::album::AlbumResponse>,
    #[serde(rename = "song")]
    pub song: Vec<super::song::SongResponse>,
}

/// 搜索结果3（包含更多详情）
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult3 {
    #[serde(rename = "artist")]
    pub artist: Vec<super::artist::ArtistResponse>,
    #[serde(rename = "album")]
    pub album: Vec<super::album::AlbumResponse>,
    #[serde(rename = "song")]
    pub song: Vec<super::song::SongResponse>,
}

/// 艺术家信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistInfo {
    pub biography: Option<String>,
    pub music_brainz_id: Option<String>,
    pub last_fm_url: Option<String>,
    pub similar_artists: Option<SimilarArtists>,
}

/// 相似艺术家
#[derive(Debug, Serialize, Deserialize)]
pub struct SimilarArtists {
    #[serde(rename = "artist")]
    pub artists: Vec<super::artist::ArtistResponse>,
}

/// 歌词
#[derive(Debug, Serialize, Deserialize)]
pub struct Lyrics {
    #[serde(rename = "@artist")]
    pub artist: Option<String>,
    #[serde(rename = "@title")]
    pub title: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

/// 聊天消息
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessages {
    #[serde(rename = "message")]
    pub messages: Vec<ChatMessage>,
}

/// 聊天消息详情
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@username")]
    pub username: String,
    #[serde(rename = "@message")]
    pub message: String,
    #[serde(rename = "@time")]
    pub time: i64,
}

/// 正在播放
#[derive(Debug, Serialize, Deserialize)]
pub struct NowPlaying {
    #[serde(rename = "entry")]
    pub entries: Vec<NowPlayingEntry>,
}

/// 正在播放条目
#[derive(Debug, Serialize, Deserialize)]
pub struct NowPlayingEntry {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@artist")]
    pub artist: String,
    #[serde(rename = "@username")]
    pub username: String,
    #[serde(rename = "@minutesAgo")]
    pub minutes_ago: i32,
}

/// 视频
#[derive(Debug, Serialize, Deserialize)]
pub struct Videos {
    #[serde(rename = "video")]
    pub videos: Vec<Video>,
}

/// 视频详情
#[derive(Debug, Serialize, Deserialize)]
pub struct Video {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@title")]
    pub title: String,
    #[serde(rename = "@contentType")]
    pub content_type: String,
}

/// 视频信息
#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@title")]
    pub title: String,
    pub captions: Option<Captions>,
}

/// 字幕
#[derive(Debug, Serialize, Deserialize)]
pub struct Captions {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@label")]
    pub label: String,
}

/// HLS 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct Hls {
    pub url: String,
}

// 通用响应类型
pub type StatusResponse = SubsonicResponse<()>;

impl<T> SubsonicResponse<T> {
    pub fn ok(data: Option<T>) -> Self {
        Self {
            response: ResponseContainer {
                status: "ok".to_string(),
                version: "1.16.1".to_string(),
                error: None,
                data,
            },
        }
    }

    pub fn failed(code: i32, message: String) -> Self {
        Self {
            response: ResponseContainer {
                status: "failed".to_string(),
                version: "1.16.1".to_string(),
                error: Some(SubsonicError { code, message }),
                data: None,
            },
        }
    }
}