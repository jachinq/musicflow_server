//! 流媒体端点处理器
#![allow(dead_code)]

use axum::body::Body;
use axum::{extract::Query, http::HeaderMap, response::IntoResponse, routing::get, Router};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::error::AppError;
use crate::extractors::Format;
use crate::middleware::auth_middleware;
use crate::models::response::{Lyrics, LyricsResponse};
use crate::response::ApiResponse;
use crate::services::ServiceContext;
use crate::utils::{image_utils, MetaClient};

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
}

/// GET /rest/stream - 流式播放音乐
pub async fn stream(
    axum::extract::State(state): axum::extract::State<StreamState>,
    Query(params): Query<StreamParams>,
) -> Result<impl IntoResponse, AppError> {
    // 根据ID查询歌曲信息
    let song = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT file_path, content_type FROM songs WHERE id = ?",
    )
    .bind(&params.id)
    .fetch_optional(&state.ctx.pool)
    .await?;

    let (file_path_str, content_type) = song.ok_or_else(|| AppError::not_found("Song"))?;

    let file_path = PathBuf::from(&file_path_str);

    // 检查文件是否存在
    if !file_path.exists() {
        return Err(AppError::not_found("Audio file"));
    }

    // 打开文件
    let file = File::open(&file_path).await.map_err(AppError::IoError)?;

    // 创建流
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // 设置响应头
    let mut headers = HeaderMap::new();
    let content_type = content_type.unwrap_or_else(|| "audio/mpeg".to_string());
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
    claims: auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<StreamState>,
    Query(params): Query<DownloadParams>,
) -> Result<impl IntoResponse, AppError> {
    // 检查下载权限
    let permissions = auth_middleware::get_user_permissions(&state.ctx.pool, &claims.sub)
        .await
        .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_download() {
        return Err(AppError::access_denied("Download permission required"));
    }

    // 根据ID查询歌曲信息
    let song =
        sqlx::query_as::<_, (String, String)>("SELECT file_path, title FROM songs WHERE id = ?")
            .bind(&params.id)
            .fetch_optional(&state.ctx.pool)
            .await?;

    let (file_path_str, title) = song.ok_or_else(|| AppError::not_found("Song"))?;

    let file_path = PathBuf::from(&file_path_str);

    // 检查文件是否存在
    if !file_path.exists() {
        return Err(AppError::not_found("Audio file"));
    }

    // 打开文件
    let file = File::open(&file_path).await.map_err(AppError::IoError)?;

    // 创建流
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // 设置响应头，触发浏览器下载
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/octet-stream".parse().unwrap());
    headers.insert(
        "Content-Disposition",
        format!("attachment; filename=\"{}\"", title)
            .parse()
            .unwrap(),
    );

    Ok((headers, body).into_response())
}

/// GET /rest/getCoverArt - 获取封面图片
pub async fn get_cover_art(
    axum::extract::State(state): axum::extract::State<StreamState>,
    Query(params): Query<CoverArtParams>,
) -> Result<impl IntoResponse, AppError> {
    let cover_art_id = &params.id;
    tracing::info!("get cover art for: {}", cover_art_id);

    if cover_art_id.replace("al-", "").trim().is_empty() {
        // 返回默认数据
        tracing::warn!("Empty cover art id, return default cover art");
        return image_utils::serve_image_file(PathBuf::from("./web/default_cover.webp")).await;
    }

    let size = params.size.unwrap_or(300).clamp(50, 2000) as u32;

    // 1. 检查缓存
    let cache_path = image_utils::get_webp_cache_path(cover_art_id, size);
    if cache_path.exists() {
        return image_utils::serve_image_file(cache_path).await;
    }

    // 2. 查找原图
    match image_utils::get_original_image_path(cover_art_id) {
        Some(path) => image_utils::prewarm_cover_from_original(cover_art_id, size, &path).await?,
        None => {
            // 3. 原图不存在，尝试从网络获取
            // 3.1 从数据库查询专辑名称作为搜索关键词
            let album_name = if cover_art_id.starts_with("al-") {
                sqlx::query_as::<_, (String,)>("SELECT name FROM albums WHERE id = ?")
                    .bind(cover_art_id.replace("al-", ""))
                    .fetch_optional(&state.ctx.pool)
                    .await?
                    .map(|(name,)| name)
            } else if cover_art_id.starts_with("ar-") {
                sqlx::query_as::<_, (String,)>("SELECT name FROM artists WHERE id = ?")
                    .bind(cover_art_id.replace("ar-", ""))
                    .fetch_optional(&state.ctx.pool)
                    .await?
                    .map(|(name,)| name)
            } else {
                None
            };

            if let Some(keyword) = album_name {
                tracing::info!("Fetching cover from network for album: {}", keyword);

                // 3.2 从酷狗获取封面图片
                let response = if cover_art_id.starts_with("al-") {
                    MetaClient::new()
                        .get_kugou_album_cover_stream(&keyword)
                        .await
                        .map_err(|e| AppError::NotFound(format!("Failed to fetch cover: {}", e)))?
                } else if cover_art_id.starts_with("ar-") {
                    MetaClient::new()
                        .get_kugou_artist_cover_stream(&keyword)
                        .await
                        .map_err(|e| AppError::NotFound(format!("Failed to fetch cover: {}", e)))?
                } else {
                    return Err(AppError::not_found("Invalid cover art id"));
                };

                // 3.3 读取响应字节流
                let bytes = response.bytes().await.map_err(|e| {
                    AppError::IoError(std::io::Error::other(format!(
                        "Failed to read response bytes: {}",
                        e
                    )))
                })?;

                // 3.4 缓存到本地 ./coverArt/originals/{cover_art_id}.jpg
                let originals_dir = PathBuf::from("./coverArt/originals");
                if !originals_dir.exists() {
                    std::fs::create_dir_all(&originals_dir).map_err(AppError::IoError)?;
                }

                let original_path = originals_dir.join(format!("{}.jpg", cover_art_id));
                tokio::fs::write(&original_path, &bytes)
                    .await
                    .map_err(AppError::IoError)?;

                tracing::info!("save original cover to: {}", original_path.display());

                // 3.5 生成 WebP 缓存
                image_utils::prewarm_cover_from_original(cover_art_id, size, &original_path)
                    .await?;
            } else {
                return Err(AppError::not_found("Album not found for cover art"));
            }
        }
    }

    // 4. 统一通过 serve_image_file 返回
    image_utils::serve_image_file(cache_path).await
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
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<StreamState>,
    Query(params): Query<LyricsParams>,
) -> Result<ApiResponse<LyricsResponse>, AppError> {
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
        .fetch_optional(&state.ctx.pool)
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
        .fetch_optional(&state.ctx.pool)
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
        .fetch_optional(&state.ctx.pool)
        .await?
    } else {
        None
    };

    // 如果找到歌曲，查询艺术家名称并返回歌词
    if let Some((lyrics, artist_id, title)) = song {
        let artist_name = sqlx::query_as::<_, (String,)>("SELECT name FROM artists WHERE id = ?")
            .bind(&artist_id)
            .fetch_optional(&state.ctx.pool)
            .await?
            .map(|(name,)| name);

        let lyrics_response = LyricsResponse {
            lyrics: Lyrics {
                artist: artist_name,
                title: Some(title),
                text: lyrics,
            },
        };

        Ok(ApiResponse::ok(Some(lyrics_response), format))
    } else {
        // 如果没有找到歌词，返回空的歌词对象
        let lyrics_response = LyricsResponse {
            lyrics: Lyrics {
                artist: params.artist,
                title: params.title,
                text: None,
            },
        };
        Ok(ApiResponse::ok(Some(lyrics_response), format))
    }
}

/// GET /rest/getAvatar - 获取用户头像
pub async fn get_avatar(_params: Query<DownloadParams>) -> Result<(HeaderMap, Vec<u8>), AppError> {
    // 简化处理：返回默认头像或 404
    // 实际应用中应该从用户表查询头像路径
    Err(AppError::not_found("Avatar"))
}

#[derive(Clone)]
pub struct StreamState {
    ctx: Arc<ServiceContext>,
}
impl StreamState {
    /// 创建新的 BrowsingService
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }
}
pub fn routes() -> Router<StreamState> {
    Router::new()
        .route("/rest/stream", get(stream))
        .route("/rest/download", get(download))
        .route("/rest/getCoverArt", get(get_cover_art))
        .route("/rest/getLyrics", get(get_lyrics))
        .route("/rest/getAvatar", get(get_avatar))
}
