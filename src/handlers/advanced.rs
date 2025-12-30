//! 高级功能端点处理器
//! 包括: getNowPlaying, getSystemInfo, 聊天, 视频等
#![allow(dead_code)]

use axum::{
    Router,
    routing::{get, post},
    extract::Query,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::response::{
    NowPlaying, NowPlayingEntry,
    ChatMessages, ChatMessage, Videos, Video, VideoInfo, Hls, ToXml,
};
use crate::extractors::Format;
use crate::response::ApiResponse;

/// 通用参数
#[derive(Debug, Deserialize)]
pub struct CommonParams {
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 聊天消息参数
#[derive(Debug, Deserialize)]
pub struct GetChatMessagesParams {
    pub since: Option<i64>,  // 时间戳
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 添加聊天消息参数
#[derive(Debug, Deserialize)]
pub struct AddChatMessageParams {
    pub message: String,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 视频参数
#[derive(Debug, Deserialize)]
pub struct GetVideosParams {
    pub size: Option<i32>,
    pub offset: Option<i32>,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 视频信息参数
#[derive(Debug, Deserialize)]
pub struct GetVideoInfoParams {
    pub id: String,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// HLS 参数
#[derive(Debug, Deserialize)]
pub struct HlsParams {
    pub id: String,
    pub bit_rate: Option<Vec<i32>>,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 系统信息响应
#[derive(Debug, Clone, Serialize)]
pub struct SystemInfoResponse {
    #[serde(rename = "musicFolders")]
    pub music_folders: MusicFolders,
    #[serde(rename = "indexing")]
    pub indexing: bool,
    #[serde(rename = "scanDate")]
    pub scan_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MusicFolders {
    #[serde(rename = "musicFolder")]
    pub music_folder: Vec<MusicFolder>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MusicFolder {
    pub id: String,
    pub name: String,
}

/// GET /rest/getNowPlaying - 获取正在播放列表
pub async fn get_now_playing(
    Format(format): Format,
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(_params): Query<CommonParams>,
) -> Result<ApiResponse<NowPlaying>, AppError> {
    // 查询最近15分钟内播放的歌曲
    let now = chrono::Utc::now();
    let fifteen_minutes_ago = now - chrono::Duration::minutes(15);

    let entries = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT np.id, s.title, ar.name as artist, np.username, np.started_at
         FROM now_playing np
         JOIN songs s ON np.song_id = s.id
         JOIN artists ar ON s.artist_id = ar.id
         WHERE np.started_at >= ?
         ORDER BY np.started_at DESC"
    )
    .bind(fifteen_minutes_ago.timestamp())
    .fetch_all(&*pool)
    .await?;

    let now_playing_entries = entries
        .into_iter()
        .map(|(id, title, artist, username, started_at)| {
            let started_timestamp = started_at.parse::<i64>().unwrap_or(0);
            let minutes_ago = ((now.timestamp() - started_timestamp) / 60) as i32;

            NowPlayingEntry {
                id,
                title,
                artist,
                username,
                minutes_ago,
            }
        })
        .collect();

    let result = NowPlaying {
        entries: now_playing_entries,
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getSystemInfo - 获取系统信息
pub async fn get_system_info(
    Format(format): Format,
    axum::extract::State(_pool): axum::extract::State<Arc<SqlitePool>>,
    Query(_params): Query<CommonParams>,
) -> Result<ApiResponse<SystemInfoResponse>, AppError> {
    // 获取音乐库统计
    let music_folder = MusicFolder {
        id: "1".to_string(),
        name: "Music".to_string(),
    };

    let result = SystemInfoResponse {
        music_folders: MusicFolders {
            music_folder: vec![music_folder],
        },
        indexing: false,
        scan_date: Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getChatMessages - 获取聊天消息
pub async fn get_chat_messages(
    Format(format): Format,
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<GetChatMessagesParams>,
) -> Result<ApiResponse<ChatMessages>, AppError> {
    let since = params.since.unwrap_or(0);

    let messages = sqlx::query_as::<_, (String, String, String, i64)>(
        "SELECT id, username, message, timestamp
         FROM chat_messages
         WHERE timestamp > ?
         ORDER BY timestamp DESC
         LIMIT 100"
    )
    .bind(since)
    .fetch_all(&*pool)
    .await?;

    let chat_messages = messages
        .into_iter()
        .map(|(id, username, message, time)| ChatMessage {
            id,
            username,
            message,
            time,
        })
        .collect();

    let result = ChatMessages {
        messages: chat_messages,
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// POST /rest/addChatMessage - 添加聊天消息
pub async fn add_chat_message(
    Format(format): Format,
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<AddChatMessageParams>,
) -> Result<ApiResponse<()>, AppError> {
    let user_id = &claims.sub;

    // 获取用户名
    let username = sqlx::query_scalar::<_, String>(
        "SELECT username FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(&*pool)
    .await?
    .ok_or_else(|| AppError::not_found("User"))?;

    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO chat_messages (id, user_id, username, message, timestamp)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(user_id)
    .bind(&username)
    .bind(&params.message)
    .bind(timestamp)
    .execute(&*pool)
    .await?;

    Ok(ApiResponse::ok(None, format))
}

/// GET /rest/getVideos - 获取视频列表
pub async fn get_videos(
    Format(format): Format,
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<GetVideosParams>,
) -> Result<ApiResponse<Videos>, AppError> {
    let size = params.size.unwrap_or(20).min(500);
    let offset = params.offset.unwrap_or(0);

    let videos = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, title, content_type
         FROM videos
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(size)
    .bind(offset)
    .fetch_all(&*pool)
    .await?;

    let video_list = videos
        .into_iter()
        .map(|(id, title, content_type)| Video {
            id,
            title,
            content_type,
        })
        .collect();

    let result = Videos {
        videos: video_list,
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getVideoInfo - 获取视频信息
pub async fn get_video_info(
    Format(format): Format,
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<GetVideoInfoParams>,
) -> Result<ApiResponse<VideoInfo>, AppError> {
    let video = sqlx::query_as::<_, (String, String)>(
        "SELECT id, title FROM videos WHERE id = ?"
    )
    .bind(&params.id)
    .fetch_optional(&*pool)
    .await?
    .ok_or_else(|| AppError::not_found("Video"))?;

    let result = VideoInfo {
        id: video.0,
        title: video.1,
        captions: None,
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/hls.m3u8 - 获取 HLS 流
pub async fn hls(
    Format(format): Format,
    axum::extract::State(_pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<HlsParams>,
) -> Result<ApiResponse<Hls>, AppError> {
    // HLS 流需要生成 m3u8 播放列表
    // 这里简化处理，返回一个 URL
    let url = std::format!("/rest/stream?id={}", params.id);

    let result = Hls { url };

    Ok(ApiResponse::ok(Some(result), format))
}

pub fn routes() -> Router<Arc<SqlitePool>> {
    Router::new()
        .route("/rest/getNowPlaying", get(get_now_playing))
        .route("/rest/getSystemInfo", get(get_system_info))
        .route("/rest/getChatMessages", get(get_chat_messages))
        .route("/rest/addChatMessage", post(add_chat_message))
        .route("/rest/getVideos", get(get_videos))
        .route("/rest/getVideoInfo", get(get_video_info))
        .route("/rest/hls.m3u8", get(hls))
}

// ============================================================================
// ToXml 实现
// ============================================================================

/// SystemInfoResponse ToXml 实现
impl ToXml for SystemInfoResponse {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<systemInfo>");

        // musicFolders
        xml.push_str(&self.music_folders.to_xml_element());

        // indexing
        xml.push_str(&format!(
            r#"<indexing>{}</indexing>"#,
            self.indexing
        ));

        // scanDate
        if let Some(scan_date) = &self.scan_date {
            xml.push_str(&format!(
                r#"<scanDate>{}</scanDate>"#,
                scan_date
            ));
        }

        xml.push_str("</systemInfo>");
        xml
    }
}

/// MusicFolders ToXml 实现
impl ToXml for MusicFolders {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<musicFolders>");

        for folder in &self.music_folder {
            xml.push_str(&folder.to_xml_element());
        }

        xml.push_str("</musicFolders>");
        xml
    }
}

/// MusicFolder ToXml 实现
impl ToXml for MusicFolder {
    fn to_xml_element(&self) -> String {
        format!(
            r#"<musicFolder id="{}" name="{}"/>"#,
            self.id,
            self.name
        )
    }
}

