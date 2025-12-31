//! 通用响应结构
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// XML 序列化 trait
///
/// 用于将响应数据手动序列化为 XML 元素
/// 不依赖 serde 的 @ 标记,保持 JSON 输出干净
pub trait ToXml {
    /// 将对象序列化为 XML 元素字符串
    fn to_xml_element(&self) -> String;
}

// 为空类型实现 ToXml
impl ToXml for () {
    fn to_xml_element(&self) -> String {
        String::new()
    }
}

/// Subsonic 响应容器
///
/// 支持 JSON 和 XML 两种格式:
/// - JSON: 标准 JSON 对象,字段名干净无 @ 前缀
/// - XML: 通过 to_xml() 方法手动构建,status/version 作为属性
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "subsonic-response")]
pub struct SubsonicResponse<T> {
    #[serde(rename = "subsonic-response")]
    pub response: ResponseContainer<T>,
}

/// 响应容器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseContainer<T> {
    /// 响应状态: "ok" 或 "failed"
    pub status: String,

    /// API 版本号
    pub version: String,

    /// 错误信息 (仅在 status="failed" 时存在)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SubsonicError>,

    /// 实际响应数据
    #[serde(flatten)]
    pub data: Option<T>,
}

/// Subsonic 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsonicError {
    pub code: i32,
    pub message: String,
}

/// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub server: String,
    pub r#type: String,
}

/// 许可证信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub valid: bool,
    pub email: String,
    pub key: String,
}

/// 扫描状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStatus {
    pub scanning: bool,
    pub count: i32,
}

/// 目录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub child: Vec<Child>,
}

/// 目录子项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Child {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i32>,
}

/// 艺术家信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistInfo {
    pub biography: Option<String>,
    pub music_brainz_id: Option<String>,
    pub last_fm_url: Option<String>,
    pub similar_artists: Option<SimilarArtists>,
}

/// 相似艺术家
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarArtists {
    #[serde(rename = "artist")]
    pub artists: Vec<super::ArtistResponse>,
}

/// 歌词
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsResponse {
    pub lyrics: Lyrics,
}

impl ToXml for LyricsResponse {
    fn to_xml_element(&self) -> String {
        self.lyrics.to_xml_element()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lyrics {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub text: Option<String>,
}

impl ToXml for Lyrics {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<lyrics");

        if let Some(artist) = &self.artist {
            xml.push_str(&format!(r#" artist="{}""#, html_escape(artist)));
        }
        if let Some(title) = &self.title {
            xml.push_str(&format!(r#" title="{}""#, html_escape(title)));
        }

        if let Some(text) = &self.text {
            xml.push('>');
            xml.push_str(&html_escape(text));
            xml.push_str("</lyrics>");
        } else {
            xml.push_str("/>");
        }

        xml
    }
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessages {
    #[serde(rename = "message")]
    pub messages: Vec<ChatMessage>,
}

/// 聊天消息详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub username: String,
    pub message: String,
    pub time: i64,
}

/// 正在播放
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowPlaying {
    #[serde(rename = "entry")]
    pub entries: Vec<NowPlayingEntry>,
}

/// 正在播放条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowPlayingEntry {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub username: String,
    pub minutes_ago: i32,
}

/// 视频
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Videos {
    #[serde(rename = "video")]
    pub videos: Vec<Video>,
}

/// 视频详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub content_type: String,
}

/// 视频信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub captions: Option<Captions>,
}

/// 字幕
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Captions {
    pub id: String,
    pub label: String,
}

/// HLS 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hls {
    pub url: String,
}

// ============================================================================
// ToXml 实现
// ============================================================================

/// NowPlaying ToXml 实现
impl ToXml for NowPlaying {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<nowPlaying>");

        for entry in &self.entries {
            xml.push_str(&entry.to_xml_element());
        }

        xml.push_str("</nowPlaying>");
        xml
    }
}

/// NowPlayingEntry ToXml 实现
impl ToXml for NowPlayingEntry {
    fn to_xml_element(&self) -> String {
        format!(
            r#"<entry id="{}" title="{}" artist="{}" username="{}" minutesAgo="{}"/>"#,
            html_escape(&self.id),
            html_escape(&self.title),
            html_escape(&self.artist),
            html_escape(&self.username),
            self.minutes_ago
        )
    }
}

/// ChatMessages ToXml 实现
impl ToXml for ChatMessages {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<chatMessages>");

        for message in &self.messages {
            xml.push_str(&message.to_xml_element());
        }

        xml.push_str("</chatMessages>");
        xml
    }
}

/// ChatMessage ToXml 实现
impl ToXml for ChatMessage {
    fn to_xml_element(&self) -> String {
        format!(
            r#"<chatMessage username="{}" time="{}" message="{}"/>"#,
            html_escape(&self.username),
            self.time,
            html_escape(&self.message)
        )
    }
}

/// Videos ToXml 实现
impl ToXml for Videos {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<videos>");

        for video in &self.videos {
            xml.push_str(&video.to_xml_element());
        }

        xml.push_str("</videos>");
        xml
    }
}

/// Video ToXml 实现
impl ToXml for Video {
    fn to_xml_element(&self) -> String {
        format!(
            r#"<video id="{}" title="{}" contentType="{}"/>"#,
            html_escape(&self.id),
            html_escape(&self.title),
            html_escape(&self.content_type)
        )
    }
}

/// VideoInfo ToXml 实现
impl ToXml for VideoInfo {
    fn to_xml_element(&self) -> String {
        let mut xml = format!(
            r#"<videoInfo id="{}" title="{}""#,
            html_escape(&self.id),
            html_escape(&self.title)
        );

        if let Some(captions) = &self.captions {
            xml.push('>');
            xml.push_str(&captions.to_xml_element());
            xml.push_str("</videoInfo>");
        } else {
            xml.push_str("/>");
        }

        xml
    }
}

/// Captions ToXml 实现
impl ToXml for Captions {
    fn to_xml_element(&self) -> String {
        format!(
            r#"<captions id="{}" label="{}"/>"#,
            html_escape(&self.id),
            html_escape(&self.label)
        )
    }
}

/// Hls ToXml 实现
impl ToXml for Hls {
    fn to_xml_element(&self) -> String {
        format!(r#"<hls url="{}"/>"#, html_escape(&self.url))
    }
}

// 通用响应类型
pub type StatusResponse = SubsonicResponse<()>;

impl<T> SubsonicResponse<T> {
    /// 创建成功响应 (JSON 格式)
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

    /// 创建成功响应 (XML 格式,带命名空间)
    pub fn ok_xml(data: Option<T>) -> Self {
        Self {
            response: ResponseContainer {
                status: "ok".to_string(),
                version: "1.16.1".to_string(),
                error: None,
                data,
            },
        }
    }

    /// 创建失败响应 (JSON 格式)
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

    /// 创建失败响应 (XML 格式,带命名空间)
    pub fn failed_xml(code: i32, message: String) -> Self {
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

impl<T: ToXml> SubsonicResponse<T> {
    /// 序列化为 XML 字符串
    ///
    /// 使用手动构建的方式确保符合 Subsonic API 规范:
    /// - XML 声明
    /// - 命名空间作为属性
    /// - status 和 version 作为属性
    /// - 数据通过 ToXml trait 序列化
    pub fn to_xml(&self) -> String {
        let response = &self.response;

        // 手动构建 XML 根元素
        let mut xml = String::from("<subsonic-response xmlns=\"http://subsonic.org/restapi\"");
        xml.push_str(&format!(" status=\"{}\"", response.status));
        xml.push_str(&format!(" version=\"{}\"", response.version));
        xml.push_str(" serverVersion=\"1.0.0\"");
        // xml.push_str(" openSubsonic=true");

        // 如果有 data,添加子元素
        if let Some(ref data) = response.data {
            xml.push('>');
            xml.push_str(&data.to_xml_element());
            xml.push_str("</subsonic-response>");
        } else if let Some(ref error) = response.error {
            xml.push('>');
            xml.push_str(&format!(
                "<error code=\"{}\" message=\"{}\"/>",
                error.code,
                html_escape(&error.message)
            ));
            xml.push_str("</subsonic-response>");
        } else {
            // 空响应
            xml.push_str(
                "></subsonic-response
>",
            );
        }

        // format!(r#"<?xmlversion="1.0" encoding="UTF-8"?>

        // {}
        // "#, xml)
        xml
    }
}

/// HTML/XML 转义辅助函数
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ========== 额外的 ToXml 实现 ==========

impl ToXml for Directory {
    fn to_xml_element(&self) -> String {
        let mut xml = format!(r#"<directory id="{}" name="{}""#, self.id, self.name);
        if let Some(parent) = &self.parent {
            xml.push_str(&format!(r#" parent="{}""#, parent));
        }
        xml.push('>');
        for child in &self.child {
            xml.push_str(&child.to_xml_element());
        }
        xml.push_str("</directory>");
        xml
    }
}

impl ToXml for Child {
    fn to_xml_element(&self) -> String {
        let mut xml = format!(
            r#"<child id="{}" title="{}" isDir="{}""#,
            self.id, self.title, self.is_dir
        );
        if let Some(artist) = &self.artist {
            xml.push_str(&format!(r#" artist="{}""#, artist));
        }
        if let Some(album) = &self.album {
            xml.push_str(&format!(r#" album="{}""#, album));
        }
        if let Some(cover_art) = &self.cover_art {
            xml.push_str(&format!(r#" coverArt="{}""#, cover_art));
        }
        if let Some(duration) = self.duration {
            xml.push_str(&format!(r#" duration="{}""#, duration));
        }
        if let Some(play_count) = self.play_count {
            xml.push_str(&format!(r#" playCount="{}""#, play_count));
        }
        xml.push_str("/>");
        xml
    }
}

impl ToXml for ArtistInfo {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<artistInfo");
        if let Some(mbid) = &self.music_brainz_id {
            xml.push_str(&format!(r#" musicBrainzId="{}""#, mbid));
        }
        if let Some(url) = &self.last_fm_url {
            xml.push_str(&format!(r#" lastFmUrl="{}""#, url));
        }
        xml.push('>');
        if let Some(bio) = &self.biography {
            xml.push_str(&format!("<biography>{}</biography>", html_escape(bio)));
        }
        if let Some(similar) = &self.similar_artists {
            xml.push_str(&similar.to_xml_element());
        }
        xml.push_str("</artistInfo>");
        xml
    }
}

impl ToXml for SimilarArtists {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<similarArtists>");
        for artist in &self.artists {
            xml.push_str(&artist.to_xml_element());
        }
        xml.push_str("</similarArtists>");
        xml
    }
}
