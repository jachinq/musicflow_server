//! 播放列表端点处理器

use axum::{
    extract::Query,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{error::AppError, middleware::auth_middleware};
use crate::extractors::Format;
use crate::models::dto::{CreatePlaylistRequest, UpdatePlaylistRequest};
use crate::models::response::{PlaylistDetail, PlaylistResponse, Playlists, Song};
use crate::response::ApiResponse;
use crate::services::PlaylistService;

/// 播放列表处理器状态
#[derive(Clone)]
pub struct PlaylistState {
    pub playlist_service: Arc<PlaylistService>,
    pub pool: Arc<sqlx::SqlitePool>,
}

/// 通用播放列表参数
#[derive(Debug, Deserialize)]
pub struct PlaylistParams {
    pub id: Option<String>,
}

/// GET /rest/getPlaylists - 获取所有播放列表
pub async fn get_playlists(
    claims: auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(_params): Query<PlaylistParams>,
    Format(format): Format,
) -> Result<ApiResponse<Playlists>, AppError> {
    // 认证中间件获取当前用户名
    let username = &claims.username;

    // 调用 Service 层
    let playlists = state.playlist_service.get_playlists(username).await?;

    // 转换为响应格式
    let playlist_responses = playlists
        .into_iter()
        .map(|p| PlaylistResponse {
            id: p.id,
            name: p.name,
            owner: p.owner_id,
            public: p.is_public,
            song_count: p.song_count,
            duration: Some(p.duration),
        })
        .collect();

    let result = Playlists {
        playlists: playlist_responses,
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getPlaylist - 获取播放列表详情
pub async fn get_playlist(
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(params): Query<PlaylistParams>,
    Format(format): Format,
) -> Result<ApiResponse<PlaylistDetail>, AppError> {
    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 调用 Service 层
    let detail = state
        .playlist_service
        .get_playlist_detail(&playlist_id)
        .await?;

    // 转换为响应格式
    let result = PlaylistDetail {
        id: detail.id,
        name: detail.name,
        owner: detail.owner_id,
        public: detail.is_public,
        song_count: detail.song_count,
        duration: detail.duration,
        entry: Song::from_dtos(detail.songs),
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// POST /rest/createPlaylist - 创建播放列表
pub async fn create_playlist(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(body): Query<CreatePlaylistRequest>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    // 检查播放列表权限
    let permissions =
        crate::middleware::auth_middleware::get_user_permissions(&state.pool, &claims.sub)
            .await
            .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    // 调用 Service 层 (带事务保护)
    state
        .playlist_service
        .create_playlist(&claims.sub, body)
        .await?;

    Ok(ApiResponse::ok(None, format))
}

/// POST /rest/updatePlaylist - 更新播放列表
pub async fn update_playlist(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(params): Query<PlaylistParams>,
    Query(body): Query<UpdatePlaylistRequest>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    // 检查播放列表权限
    let permissions =
        crate::middleware::auth_middleware::get_user_permissions(&state.pool, &claims.sub)
            .await
            .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 调用 Service 层 (带事务保护,包含权限检查)
    state
        .playlist_service
        .update_playlist(&playlist_id, &claims.sub, body)
        .await?;

    Ok(ApiResponse::ok(None, format))
}

/// POST /rest/deletePlaylist - 删除播放列表
pub async fn delete_playlist(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(params): Query<PlaylistParams>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    // 检查播放列表权限
    let permissions =
        crate::middleware::auth_middleware::get_user_permissions(&state.pool, &claims.sub)
            .await
            .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 调用 Service 层 (包含权限检查)
    state
        .playlist_service
        .delete_playlist(&playlist_id, &claims.sub)
        .await?;

    Ok(ApiResponse::ok(None, format))
}

/// POST /rest/appendPlaylist - 追加歌曲到播放列表
pub async fn append_playlist(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(params): Query<PlaylistParams>,
    Query(body): Query<CreatePlaylistRequest>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    // 检查播放列表权限
    let permissions =
        crate::middleware::auth_middleware::get_user_permissions(&state.pool, &claims.sub)
            .await
            .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 如果有歌曲,追加到播放列表
    if let Some(song_ids) = body.song_id {
        // 调用 Service 层 (带事务保护,包含权限检查)
        state
            .playlist_service
            .append_songs(&playlist_id, &claims.sub, song_ids)
            .await?;
    }

    Ok(ApiResponse::ok(None, format))
}

pub fn routes() -> Router<PlaylistState> {
    Router::new()
        .route("/rest/getPlaylists", get(get_playlists))
        .route("/rest/getPlaylist", get(get_playlist))
        .route("/rest/createPlaylist", post(create_playlist))
        .route("/rest/updatePlaylist", post(update_playlist))
        .route("/rest/deletePlaylist", post(delete_playlist))
        .route("/rest/appendPlaylist", post(append_playlist))
}
