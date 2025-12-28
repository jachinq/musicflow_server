//! 播放列表端点处理器

use axum::{extract::Query, routing::{get, post}, Json, Router};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::models::dto::{CreatePlaylistRequest, UpdatePlaylistRequest};
use crate::models::response::{
    PlaylistDetail, PlaylistResponse, Playlists, ResponseContainer, Song, SubsonicResponse,
};
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
    pub u: String,
}

/// GET /rest/getPlaylists - 获取所有播放列表
pub async fn get_playlists(
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(params): Query<PlaylistParams>,
) -> Result<Json<SubsonicResponse<Playlists>>, AppError> {
    // 调用 Service 层
    let playlists = state.playlist_service.get_playlists(&params.u).await?;

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

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: Some(result),
        },
    }))
}

/// GET /rest/getPlaylist - 获取播放列表详情
pub async fn get_playlist(
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(params): Query<PlaylistParams>,
) -> Result<Json<SubsonicResponse<PlaylistDetail>>, AppError> {
    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 调用 Service 层
    let detail = state.playlist_service.get_playlist_detail(&playlist_id).await?;

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

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: Some(result),
        },
    }))
}

/// POST /rest/createPlaylist - 创建播放列表
pub async fn create_playlist(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(params): Query<PlaylistParams>,
    Json(body): Json<CreatePlaylistRequest>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 检查播放列表权限
    let permissions = crate::middleware::auth_middleware::get_user_permissions(&state.pool, &claims.sub)
        .await
        .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    // 调用 Service 层 (带事务保护)
    state.playlist_service.create_playlist(&params.u, body).await?;

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: None,
        },
    }))
}

/// POST /rest/updatePlaylist - 更新播放列表
pub async fn update_playlist(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(params): Query<PlaylistParams>,
    Json(body): Json<UpdatePlaylistRequest>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 检查播放列表权限
    let permissions = crate::middleware::auth_middleware::get_user_permissions(&state.pool, &claims.sub)
        .await
        .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 调用 Service 层 (带事务保护,包含权限检查)
    state.playlist_service
        .update_playlist(&playlist_id, &params.u, body)
        .await?;

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: None,
        },
    }))
}

/// POST /rest/deletePlaylist - 删除播放列表
pub async fn delete_playlist(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(params): Query<PlaylistParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 检查播放列表权限
    let permissions = crate::middleware::auth_middleware::get_user_permissions(&state.pool, &claims.sub)
        .await
        .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 调用 Service 层 (包含权限检查)
    state.playlist_service
        .delete_playlist(&playlist_id, &params.u)
        .await?;

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: None,
        },
    }))
}

/// POST /rest/appendPlaylist - 追加歌曲到播放列表
pub async fn append_playlist(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<PlaylistState>,
    Query(params): Query<PlaylistParams>,
    Json(body): Json<CreatePlaylistRequest>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 检查播放列表权限
    let permissions = crate::middleware::auth_middleware::get_user_permissions(&state.pool, &claims.sub)
        .await
        .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 如果有歌曲,追加到播放列表
    if let Some(song_ids) = body.song_id {
        // 调用 Service 层 (带事务保护,包含权限检查)
        state.playlist_service
            .append_songs(&playlist_id, &params.u, song_ids)
            .await?;
    }

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: None,
        },
    }))
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
