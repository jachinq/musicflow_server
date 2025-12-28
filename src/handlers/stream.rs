//! 流媒体端点处理器
#![allow(dead_code)]

use axum::body::Body;
use axum::{extract::Query, http::HeaderMap, response::IntoResponse, routing::get, Json, Router};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::error::AppError;
use crate::models::response::{Lyrics, LyricsResponse, SubsonicResponse};

/// 流媒体参数
#[derive(Debug, Deserialize)]
pub struct StreamParams {
    pub id: String,
    #[serde(rename = "maxBitRate")]
    pub max_bit_rate: Option<i32>,
    pub format: Option<String>,
    #[serde(rename = "sampleRate")]
    pub time_offset: Option<i32>,
    #[serde(rename = "startTime")]
    pub estimate_content_length: Option<bool>,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 下载参数
#[derive(Debug, Deserialize)]
pub struct DownloadParams {
    pub id: String,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 获取封面参数
#[derive(Debug, Deserialize)]
pub struct CoverArtParams {
    pub id: String,
    pub size: Option<i32>,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// GET /rest/stream - 流式播放音乐
pub async fn stream(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<StreamParams>,
) -> Result<impl IntoResponse, AppError> {
    // 根据ID查询歌曲信息
    let song = sqlx::query!(
        "SELECT file_path, content_type FROM songs WHERE id = ?",
        params.id
    )
    .fetch_optional(&*pool)
    .await?;

    let song = song.ok_or_else(|| AppError::not_found("Song"))?;

    let file_path = PathBuf::from(&song.file_path);

    // 检查文件是否存在
    if !file_path.exists() {
        return Err(AppError::not_found("Audio file"));
    }

    // 打开文件
    let file = File::open(&file_path)
        .await
        .map_err(|e| AppError::IoError(e))?;

    // 创建流
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // 设置响应头
    let mut headers = HeaderMap::new();
    let content_type = song
        .content_type
        .unwrap_or_else(|| "audio/mpeg".to_string());
    headers.insert("Content-Type", content_type.parse().unwrap());
    headers.insert("Accept-Ranges", "bytes".parse().unwrap());

    // 如果估计内容长度，可以设置 Content-Length
    if params.estimate_content_length.unwrap_or(false) {
        if let Ok(metadata) = tokio::fs::metadata(&file_path).await {
            headers.insert(
                "Content-Length",
                metadata.len().to_string().parse().unwrap(),
            );
        }
    }

    Ok((headers, body).into_response())
}

/// GET /rest/download - 下载音乐文件
pub async fn download(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<DownloadParams>,
) -> Result<impl IntoResponse, AppError> {
    // 检查下载权限
    let permissions = crate::middleware::auth_middleware::get_user_permissions(&pool, &claims.sub)
        .await
        .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_download() {
        return Err(AppError::access_denied("Download permission required"));
    }

    // 根据ID查询歌曲信息
    let song = sqlx::query!("SELECT file_path, title FROM songs WHERE id = ?", params.id)
        .fetch_optional(&*pool)
        .await?;

    let song = song.ok_or_else(|| AppError::not_found("Song"))?;

    let file_path = PathBuf::from(&song.file_path);

    // 检查文件是否存在
    if !file_path.exists() {
        return Err(AppError::not_found("Audio file"));
    }

    // 打开文件
    let file = File::open(&file_path)
        .await
        .map_err(|e| AppError::IoError(e))?;

    // 创建流
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // 设置响应头，触发浏览器下载
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/octet-stream".parse().unwrap());
    headers.insert(
        "Content-Disposition",
        format!("attachment; filename=\"{}\"", song.title)
            .parse()
            .unwrap(),
    );

    Ok((headers, body).into_response())
}

/// GET /rest/getCoverArt - 获取封面图片
pub async fn get_cover_art(
    // claims: crate::middleware::auth_middleware::Claims,
    // axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<CoverArtParams>,
) -> Result<impl IntoResponse, AppError> {
    // // 检查封面艺术权限
    // let permissions = crate::middleware::auth_middleware::get_user_permissions(&pool, &claims.sub)
    //     .await
    //     .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    // if !permissions.can_access_cover_art() {
    //     return Err(AppError::access_denied("Cover art permission required"));
    // }

    let file_path = format!("./coverArt/{}.jpg", params.id);
    let file_path = PathBuf::from(&file_path);
    if file_path.exists() {
        return serve_image_file(file_path).await;
    }

    // 如果没有找到封面，返回默认封面或 404
    // 这里可以返回一个默认的占位图
    Err(AppError::not_found("Cover art"))
}

/// 服务图片文件
async fn serve_image_file(file_path: PathBuf) -> Result<impl IntoResponse, AppError> {
    let file = File::open(&file_path)
        .await
        .map_err(|e| AppError::IoError(e))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // 根据文件扩展名确定 Content-Type
    let ext = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let content_type = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    };

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", content_type.parse().unwrap());

    Ok((headers, body).into_response())
}

/// 歌词查询参数
#[derive(Debug, Deserialize)]
pub struct LyricsParams {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// GET /rest/getLyrics - 获取歌词
pub async fn get_lyrics(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<LyricsParams>,
) -> Result<Json<SubsonicResponse<LyricsResponse>>, AppError> {
    // 构建查询条件
    let song = if let (Some(artist), Some(title)) = (params.artist.as_ref(), params.title.as_ref())
    {
        // 同时有艺术家和标题
        sqlx::query_as::<_, (Option<String>, String, String)>(
            "SELECT s.lyrics, s.artist_id, s.title
             FROM songs s
             JOIN artists a ON s.artist_id = a.id
             WHERE s.title LIKE ? AND a.name LIKE ?
             LIMIT 1",
        )
        .bind(format!("%{}%", title))
        .bind(format!("%{}%", artist))
        .fetch_optional(&*pool)
        .await?
    } else if let Some(title) = params.title.as_ref() {
        // 只有标题
        sqlx::query_as::<_, (Option<String>, String, String)>(
            "SELECT lyrics, artist_id, title
             FROM songs
             WHERE title LIKE ?
             LIMIT 1",
        )
        .bind(format!("%{}%", title))
        .fetch_optional(&*pool)
        .await?
    } else if let Some(artist) = params.artist.as_ref() {
        // 只有艺术家
        sqlx::query_as::<_, (Option<String>, String, String)>(
            "SELECT s.lyrics, s.artist_id, s.title
             FROM songs s
             JOIN artists a ON s.artist_id = a.id
             WHERE a.name LIKE ? AND s.lyrics IS NOT NULL
             LIMIT 1",
        )
        .bind(format!("%{}%", artist))
        .fetch_optional(&*pool)
        .await?
    } else {
        None
    };

    // 如果找到歌曲，查询艺术家名称并返回歌词
    if let Some((lyrics, artist_id, title)) = song {
        let artist_name = sqlx::query_scalar::<_, String>("SELECT name FROM artists WHERE id = ?")
            .bind(&artist_id)
            .fetch_optional(&*pool)
            .await?;

        let lyrics_response = LyricsResponse {
            lyrics: Lyrics {
                artist: artist_name,
                title: Some(title),
                text: lyrics,
            },
        };

        Ok(Json(SubsonicResponse::ok(Some(lyrics_response))))
    } else {
        // 如果没有找到歌词，返回空的歌词对象
        let lyrics_response = LyricsResponse {
            lyrics: Lyrics {
                artist: params.artist,
                title: params.title,
                text: None,
            },
        };
        Ok(Json(SubsonicResponse::ok(Some(lyrics_response))))
    }
}

/// GET /rest/getAvatar - 获取用户头像
pub async fn get_avatar(
    axum::extract::State(_pool): axum::extract::State<Arc<SqlitePool>>,
    _params: Query<DownloadParams>,
) -> Result<(HeaderMap, Vec<u8>), AppError> {
    // 简化处理：返回默认头像或 404
    // 实际应用中应该从用户表查询头像路径
    Err(AppError::not_found("Avatar"))
}

pub fn routes() -> Router<Arc<SqlitePool>> {
    Router::new()
        .route("/rest/stream", get(stream))
        .route("/rest/download", get(download))
        .route("/rest/getCoverArt", get(get_cover_art))
        .route("/rest/getLyrics", get(get_lyrics))
        .route("/rest/getAvatar", get(get_avatar))
}
