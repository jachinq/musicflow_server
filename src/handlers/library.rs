//! 库管理端点处理器

use axum::{
    Router,
    routing::{get, post},
    extract::Query,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::error::AppError;
use crate::services::ScanService;
use crate::models::response::{SubsonicResponse, ResponseContainer};

/// 扫描状态
#[derive(Debug, Serialize)]
pub struct ScanStatusResponse {
    scanning: bool,
    count: i32,
}

/// 扫描请求参数
#[derive(Debug, Deserialize)]
pub struct ScanParams {
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 收藏参数
#[derive(Debug, Deserialize)]
pub struct StarParams {
    pub id: String,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub song_id: Option<String>,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 评分参数
#[derive(Debug, Deserialize)]
pub struct SetRatingParams {
    pub id: String,
    pub rating: i32,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 获取评分参数
#[derive(Debug, Deserialize)]
pub struct GetRatingParams {
    pub id: String,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// Scrobble 参数
#[derive(Debug, Deserialize)]
pub struct ScrobbleParams {
    pub id: String,
    pub submission: Option<bool>,
    pub time: Option<i64>,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 扫描状态（共享）
#[derive(Clone)]
pub struct ScanState {
    pub scanning: Arc<Mutex<bool>>,
}

/// 组合状态，用于 library 路由
#[derive(Clone)]
pub struct LibraryState {
    pub pool: Arc<sqlx::SqlitePool>,
    pub scan_service: Arc<ScanService>,
    pub scan_state: ScanState,
}

/// GET /rest/getScanStatus
pub async fn get_scan_status(
    axum::extract::State(state): axum::extract::State<LibraryState>,
) -> Result<Json<SubsonicResponse<ScanStatusResponse>>, AppError> {
    let scanning = *state.scan_state.scanning.lock().await;

    // 获取数据库中的记录数作为count
    let count = 0; // 简化处理

    let result = ScanStatusResponse {
        scanning,
        count,
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

/// POST /rest/startScan
pub async fn start_scan(
    axum::extract::State(state): axum::extract::State<LibraryState>,
    _params: Query<ScanParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    tracing::info!("开始扫描");

    let mut scanning = state.scan_state.scanning.lock().await;

    if *scanning {
        return Err(AppError::server_busy("Scan already in progress"));
    }

    *scanning = true;

    // 启动后台扫描任务
    let service = state.scan_service.clone();
    let scan_state = state.scan_state.clone();

    tokio::spawn(async move {
        match service.scan_library().await {
            Ok(result) => {
                tracing::info!("扫描完成: {:?}", result);
            }
            Err(e) => {
                tracing::error!("扫描失败: {}", e);
            }
        }
        // 更新状态
        let mut scanning = scan_state.scanning.lock().await;
        *scanning = false;
    });

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: None,
        },
    }))
}

/// POST /rest/scrobble
pub async fn scrobble(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<ScrobbleParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 检查Scrobble权限
    let permissions = crate::middleware::auth_middleware::get_user_permissions(&state.pool, &claims.sub)
        .await
        .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_scrobble() {
        return Err(AppError::access_denied("Scrobbling permission required"));
    }

    let timestamp = params.time.unwrap_or_else(|| {
        chrono::Utc::now().timestamp()
    });
    let submission = params.submission.unwrap_or(true);

    // 从认证信息中获取用户ID
    let user_id = &claims.sub;

    sqlx::query(
        "INSERT INTO scrobbles (id, user_id, song_id, timestamp, submission, created_at)
         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(&params.id)
    .bind(timestamp)
    .bind(submission)
    .execute(&*state.pool)
    .await?;

    // 更新歌曲播放次数
    if submission {
        sqlx::query("UPDATE songs SET play_count = play_count + 1 WHERE id = ?")
            .bind(&params.id)
            .execute(&*state.pool)
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

/// POST /rest/star
pub async fn star(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<StarParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    let user_id = &claims.sub;

    if let Some(artist_id) = params.artist_id {
        sqlx::query(
            "INSERT OR IGNORE INTO starred (id, user_id, artist_id, created_at)
             VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
        )
        .bind(&id)
        .bind(user_id)
        .bind(&artist_id)
        .execute(&*state.pool)
        .await?;
    } else if let Some(album_id) = params.album_id {
        sqlx::query(
            "INSERT OR IGNORE INTO starred (id, user_id, album_id, created_at)
             VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
        )
        .bind(&id)
        .bind(user_id)
        .bind(&album_id)
        .execute(&*state.pool)
        .await?;
    } else if let Some(song_id) = params.song_id {
        sqlx::query(
            "INSERT OR IGNORE INTO starred (id, user_id, song_id, created_at)
             VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
        )
        .bind(&id)
        .bind(user_id)
        .bind(&song_id)
        .execute(&*state.pool)
        .await?;
    } else {
        return Err(AppError::missing_parameter("id or artist_id/album_id/song_id"));
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

/// POST /rest/unstar
pub async fn unstar(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<StarParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    let user_id = &claims.sub;

    if let Some(artist_id) = params.artist_id {
        sqlx::query(
            "DELETE FROM starred WHERE user_id = ? AND artist_id = ?"
        )
        .bind(user_id)
        .bind(&artist_id)
        .execute(&*state.pool)
        .await?;
    } else if let Some(album_id) = params.album_id {
        sqlx::query(
            "DELETE FROM starred WHERE user_id = ? AND album_id = ?"
        )
        .bind(user_id)
        .bind(&album_id)
        .execute(&*state.pool)
        .await?;
    } else if let Some(song_id) = params.song_id {
        sqlx::query(
            "DELETE FROM starred WHERE user_id = ? AND song_id = ?"
        )
        .bind(user_id)
        .bind(&song_id)
        .execute(&*state.pool)
        .await?;
    } else {
        return Err(AppError::missing_parameter("id or artist_id/album_id/song_id"));
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

/// POST /rest/setRating
pub async fn set_rating(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<SetRatingParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    if params.rating < 1 || params.rating > 5 {
        return Err(AppError::validation_error("Rating must be between 1 and 5"));
    }

    let user_id = &claims.sub;

    // 检查ID是艺术家、专辑还是歌曲
    // 这里简化处理，假设是歌曲ID
    sqlx::query(
        "INSERT OR REPLACE INTO ratings (id, user_id, song_id, rating, created_at, updated_at)
         VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(&params.id)
    .bind(params.rating)
    .execute(&*state.pool)
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

/// GET /rest/getRating
pub async fn get_rating(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<GetRatingParams>,
) -> Result<Json<SubsonicResponse<crate::models::rating::RatingResponse>>, AppError> {
    let user_id = &claims.sub;

    let rating = sqlx::query_scalar::<_, i32>(
        "SELECT rating FROM ratings WHERE user_id = ? AND song_id = ?"
    )
    .bind(user_id)
    .bind(&params.id)
    .fetch_optional(&*state.pool)
    .await?;

    if let Some(rating_value) = rating {
        let result = crate::models::rating::RatingResponse {
            id: params.id.clone(),
            rating: rating_value,
        };

        Ok(Json(SubsonicResponse {
            response: ResponseContainer {
                status: "ok".to_string(),
                version: "1.16.1".to_string(),
                error: None,
                data: Some(result),
            },
        }))
    } else {
        // 如果没有评分，返回默认评分0
        let result = crate::models::rating::RatingResponse {
            id: params.id.clone(),
            rating: 0,
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
}

/// GET /rest/getStarred
pub async fn get_starred(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    _params: Query<ScanParams>,
) -> Result<Json<SubsonicResponse<crate::models::starred::StarredResponse>>, AppError> {
    let user_id = &claims.sub;

    // 查询收藏的艺术家
    let starred_artists = sqlx::query_as::<_, crate::models::artist::ArtistResponse>(
        "SELECT a.id, a.name, a.cover_art_path as cover_art, 0 as album_count
         FROM starred s
         JOIN artists a ON s.artist_id = a.id
         WHERE s.user_id = ? AND s.artist_id IS NOT NULL"
    )
    .bind(user_id)
    .fetch_all(&*state.pool)
    .await?;

    // 查询收藏的专辑
    let starred_albums = sqlx::query_as::<_, crate::models::album::AlbumResponse>(
        "SELECT
            a.id,
            a.name,
            ar.name as artist,
            ar.id as artist_id,
            a.cover_art_path as cover_art,
            a.song_count,
            a.duration,
            a.play_count,
            a.year,
            a.genre
         FROM starred s
         JOIN albums a ON s.album_id = a.id
         JOIN artists ar ON a.artist_id = ar.id
         WHERE s.user_id = ? AND s.album_id IS NOT NULL"
    )
    .bind(user_id)
    .fetch_all(&*state.pool)
    .await?;

    // 查询收藏的歌曲
    let starred_songs = sqlx::query_as::<_, crate::models::song::SongResponse>(
        "SELECT
            s.id,
            s.title,
            ar.name as artist,
            al.name as album,
            s.duration,
            s.content_type
         FROM starred st
         JOIN songs s ON st.song_id = s.id
         JOIN albums al ON s.album_id = al.id
         JOIN artists ar ON s.artist_id = ar.id
         WHERE st.user_id = ? AND st.song_id IS NOT NULL"
    )
    .bind(user_id)
    .fetch_all(&*state.pool)
    .await?;

    let result = crate::models::starred::StarredResponse {
        artist: if starred_artists.is_empty() { None } else { Some(starred_artists) },
        album: if starred_albums.is_empty() { None } else { Some(starred_albums) },
        song: if starred_songs.is_empty() { None } else { Some(starred_songs) },
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

pub fn routes(
    pool: Arc<sqlx::SqlitePool>,
    scan_service: Arc<ScanService>,
) -> Router {
    let scan_state = ScanState {
        scanning: Arc::new(Mutex::new(false)),
    };

    let library_state = LibraryState {
        pool: pool.clone(),
        scan_service,
        scan_state,
    };

    Router::new()
        .route("/rest/getScanStatus", get(get_scan_status))
        .route("/rest/startScan", post(start_scan))
        .route("/rest/scrobble", post(scrobble))
        .route("/rest/star", post(star))
        .route("/rest/unstar", post(unstar))
        .route("/rest/setRating", post(set_rating))
        .route("/rest/getRating", get(get_rating))
        .route("/rest/getStarred", get(get_starred))
        .with_state(library_state)
}
