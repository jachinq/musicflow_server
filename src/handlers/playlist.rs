//! 播放列表端点处理器

use axum::{
    Router,
    routing::{get, post},
    extract::Query,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::response::{SubsonicResponse, ResponseContainer};
use crate::models::playlist::{PlaylistResponse, PlaylistDetail, Playlists, CreatePlaylistRequest, UpdatePlaylistRequest};
use crate::models::song::SongResponse;

/// 通用播放列表参数
#[derive(Debug, Deserialize)]
pub struct PlaylistParams {
    pub id: Option<String>,
    pub u: String,
}

/// GET /rest/getPlaylists - 获取所有播放列表
pub async fn get_playlists(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<PlaylistParams>,
) -> Result<Json<SubsonicResponse<Playlists>>, AppError> {
    // 获取当前用户的播放列表
    let playlists = sqlx::query_as::<_, (String, String, String, bool, i32, i32)>(
        "SELECT id, name, owner_id, is_public, song_count, duration
         FROM playlists
         WHERE owner_id = ? OR is_public = true
         ORDER BY name"
    )
    .bind(&params.u)
    .fetch_all(&*pool)
    .await?;

    let playlist_responses = playlists
        .into_iter()
        .map(|(id, name, owner, public, song_count, duration)| PlaylistResponse {
            id,
            name,
            owner,
            public,
            song_count,
            duration: Some(duration),
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
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<PlaylistParams>,
) -> Result<Json<SubsonicResponse<PlaylistDetail>>, AppError> {
    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 获取播放列表基本信息
    let playlist_info = sqlx::query_as::<_, (String, String, String, bool, i32, i32)>(
        "SELECT id, name, owner_id, is_public, song_count, duration
         FROM playlists
         WHERE id = ?"
    )
    .bind(&playlist_id)
    .fetch_optional(&*pool)
    .await?;

    let (id, name, owner, public, song_count, duration) = playlist_info
        .ok_or_else(|| AppError::not_found("Playlist"))?;

    // 获取播放列表中的歌曲
    let songs = sqlx::query_as::<_, SongResponse>(
        "SELECT s.id, s.title, ar.name as artist, al.name as album, s.duration, s.content_type
         FROM playlist_songs ps
         JOIN songs s ON ps.song_id = s.id
         JOIN albums al ON s.album_id = al.id
         JOIN artists ar ON s.artist_id = ar.id
         WHERE ps.playlist_id = ?
         ORDER BY ps.position"
    )
    .bind(&playlist_id)
    .fetch_all(&*pool)
    .await?;

    let result = PlaylistDetail {
        id,
        name,
        owner,
        public,
        song_count,
        duration,
        entry: songs,
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
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<PlaylistParams>,
    Json(body): Json<CreatePlaylistRequest>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 检查播放列表权限
    let permissions = crate::middleware::auth_middleware::get_user_permissions(&pool, &claims.sub)
        .await
        .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    let playlist_id = uuid::Uuid::new_v4().to_string();

    // 创建播放列表
    sqlx::query(
        "INSERT INTO playlists (id, owner_id, name, comment, is_public, song_count, duration, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 0, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(&playlist_id)
    .bind(&params.u)
    .bind(&body.name)
    .bind(None::<String>)
    .bind(false)
    .execute(&*pool)
    .await?;

    // 如果提供了初始歌曲，添加到播放列表
    if let Some(song_ids) = body.song_id {
        for (position, song_id) in song_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO playlist_songs (playlist_id, song_id, position)
                 VALUES (?, ?, ?)"
            )
            .bind(&playlist_id)
            .bind(song_id)
            .bind(position as i32)
            .execute(&*pool)
            .await?;
        }

        // 更新播放列表统计
        update_playlist_stats(&pool, &playlist_id).await?;
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

/// POST /rest/updatePlaylist - 更新播放列表
pub async fn update_playlist(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<PlaylistParams>,
    Json(body): Json<UpdatePlaylistRequest>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 检查播放列表权限
    let permissions = crate::middleware::auth_middleware::get_user_permissions(&pool, &claims.sub)
        .await
        .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 检查权限（必须是所有者）
    let owner = sqlx::query_scalar::<_, String>(
        "SELECT owner_id FROM playlists WHERE id = ?"
    )
    .bind(&playlist_id)
    .fetch_optional(&*pool)
    .await?;

    if let Some(owner_id) = owner {
        if owner_id != params.u {
            return Err(AppError::access_denied("Not playlist owner"));
        }
    } else {
        return Err(AppError::not_found("Playlist"));
    }

    // 更新基本信息
    if body.name.is_some() || body.comment.is_some() || body.public.is_some() {
        let mut query_parts = Vec::new();
        let mut bind_values = Vec::new();

        if let Some(name) = &body.name {
            query_parts.push("name = ?");
            bind_values.push(name.clone());
        }

        if let Some(comment) = &body.comment {
            query_parts.push("comment = ?");
            bind_values.push(comment.clone());
        }

        if let Some(public) = &body.public {
            query_parts.push("is_public = ?");
            bind_values.push(public.to_string());
        }

        if !query_parts.is_empty() {
            let query_sql = format!(
                "UPDATE playlists SET {}, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                query_parts.join(", ")
            );

            let mut query = sqlx::query(&query_sql);
            for value in &bind_values {
                query = query.bind(value);
            }
            query.bind(&playlist_id).execute(&*pool).await?;
        }
    }

    // 添加歌曲
    if let Some(song_ids) = body.song_id_to_add {
        // 获取当前歌曲数量作为位置
        let current_count = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM playlist_songs WHERE playlist_id = ?"
        )
        .bind(&playlist_id)
        .fetch_one(&*pool)
        .await?;

        for (offset, song_id) in song_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO playlist_songs (playlist_id, song_id, position)
                 VALUES (?, ?, ?)"
            )
            .bind(&playlist_id)
            .bind(song_id)
            .bind(current_count + offset as i32)
            .execute(&*pool)
            .await?;
        }
    }

    // 删除歌曲
    if let Some(indices) = body.song_index_to_remove {
        for index in indices {
            sqlx::query(
                "DELETE FROM playlist_songs
                 WHERE playlist_id = ? AND position = ?"
            )
            .bind(&playlist_id)
            .bind(index)
            .execute(&*pool)
            .await?;
        }

        // 重新排序位置
        sqlx::query(
            "UPDATE playlist_songs
             SET position = (
                 SELECT COUNT(*) FROM playlist_songs ps2
                 WHERE ps2.playlist_id = playlist_songs.playlist_id
                 AND ps2.position < playlist_songs.position
             )
             WHERE playlist_id = ?"
        )
        .bind(&playlist_id)
        .execute(&*pool)
        .await?;
    }

    // 更新统计
    update_playlist_stats(&pool, &playlist_id).await?;

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
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<PlaylistParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 检查播放列表权限
    let permissions = crate::middleware::auth_middleware::get_user_permissions(&pool, &claims.sub)
        .await
        .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 检查权限
    let owner = sqlx::query_scalar::<_, String>(
        "SELECT owner_id FROM playlists WHERE id = ?"
    )
    .bind(&playlist_id)
    .fetch_optional(&*pool)
    .await?;

    if let Some(owner_id) = owner {
        if owner_id != params.u {
            return Err(AppError::access_denied("Not playlist owner"));
        }
    } else {
        return Err(AppError::not_found("Playlist"));
    }

    // 删除播放列表（级联删除歌曲关联）
    sqlx::query("DELETE FROM playlists WHERE id = ?")
        .bind(&playlist_id)
        .execute(&*pool)
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
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<PlaylistParams>,
    Json(body): Json<CreatePlaylistRequest>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 检查播放列表权限
    let permissions = crate::middleware::auth_middleware::get_user_permissions(&pool, &claims.sub)
        .await
        .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_manage_playlist() {
        return Err(AppError::access_denied("Playlist permission required"));
    }

    let playlist_id = params.id.ok_or_else(|| AppError::missing_parameter("id"))?;

    // 检查权限
    let owner = sqlx::query_scalar::<_, String>(
        "SELECT owner_id FROM playlists WHERE id = ?"
    )
    .bind(&playlist_id)
    .fetch_optional(&*pool)
    .await?;

    if let Some(owner_id) = owner {
        if owner_id != params.u {
            return Err(AppError::access_denied("Not playlist owner"));
        }
    } else {
        return Err(AppError::not_found("Playlist"));
    }

    // 获取当前歌曲数量
    let current_count = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM playlist_songs WHERE playlist_id = ?"
    )
    .bind(&playlist_id)
    .fetch_one(&*pool)
    .await?;

    // 添加歌曲
    if let Some(song_ids) = body.song_id {
        for (offset, song_id) in song_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO playlist_songs (playlist_id, song_id, position)
                 VALUES (?, ?, ?)"
            )
            .bind(&playlist_id)
            .bind(song_id)
            .bind(current_count + offset as i32)
            .execute(&*pool)
            .await?;
        }

        // 更新统计
        update_playlist_stats(&pool, &playlist_id).await?;
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

/// 辅助函数：更新播放列表统计信息
async fn update_playlist_stats(pool: &SqlitePool, playlist_id: &str) -> Result<(), AppError> {
    // 计算歌曲数量和总时长
    let stats = sqlx::query!(
        "SELECT
            COUNT(*) as count,
            SUM(duration) as total_duration
         FROM playlist_songs ps
         JOIN songs s ON ps.song_id = s.id
         WHERE ps.playlist_id = ?",
        playlist_id
    )
    .fetch_one(pool)
    .await?;

    let song_count = stats.count;
    let duration = stats.total_duration.unwrap_or(0);

    // 更新播放列表
    sqlx::query(
        "UPDATE playlists
         SET song_count = ?, duration = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?"
    )
    .bind(song_count)
    .bind(duration)
    .bind(playlist_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub fn routes() -> Router<Arc<SqlitePool>> {
    Router::new()
        .route("/rest/getPlaylists", get(get_playlists))
        .route("/rest/getPlaylist", get(get_playlist))
        .route("/rest/createPlaylist", post(create_playlist))
        .route("/rest/updatePlaylist", post(update_playlist))
        .route("/rest/deletePlaylist", post(delete_playlist))
        .route("/rest/appendPlaylist", post(append_playlist))
}
