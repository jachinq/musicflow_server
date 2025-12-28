//! 库管理端点处理器
#![allow(dead_code)]

use crate::error::AppError;
use crate::models::response::{
    AlbumResponse, ArtistResponse, RatingResponse, ResponseContainer, Song, SubsonicResponse,
};
use crate::services::{LibraryService, ScanService, StarItemType};
use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 扫描状态
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatusResponse {
    pub scan_status: ScanStatus,
}
#[derive(Debug, Serialize)]
pub struct ScanStatus {
    scanning: bool,
    count: usize,
    current: usize,
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
    pub current: Arc<Mutex<usize>>,
    pub count: Arc<Mutex<usize>>,
}

/// 组合状态,用于 library 路由
#[derive(Clone)]
pub struct LibraryState {
    pub pool: Arc<sqlx::SqlitePool>,
    pub scan_service: Arc<ScanService>,
    pub library_service: Arc<LibraryService>,
    pub scan_state: ScanState,
}

/// GET /rest/getScanStatus
pub async fn get_scan_status(
    axum::extract::State(state): axum::extract::State<LibraryState>,
) -> Result<Json<SubsonicResponse<ScanStatusResponse>>, AppError> {
    let scanning = *state.scan_state.scanning.lock().await;
    let count = *state.scan_state.count.lock().await;
    let current = *state.scan_state.current.lock().await;

    // 从 service 层获取歌曲总数
    // let count = state.library_service.get_song_count().await?;

    let result = ScanStatusResponse {
        scan_status: ScanStatus { scanning, count, current },
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
    // 记录耗时
    let start_time = std::time::Instant::now();
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
        match service.scan_library(scan_state.clone()).await {
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
        tracing::info!("扫描结束,耗时: {}s", start_time.elapsed().as_secs());
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
    let permissions =
        crate::middleware::auth_middleware::get_user_permissions(&state.pool, &claims.sub)
            .await
            .map_err(|_| AppError::access_denied("Failed to check permissions"))?;

    if !permissions.can_scrobble() {
        return Err(AppError::access_denied("Scrobbling permission required"));
    }

    let timestamp = params
        .time
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    let submission = params.submission.unwrap_or(true);

    // 调用 Service 层 (带事务保护)
    state
        .library_service
        .submit_scrobble(&claims.sub, &params.id, timestamp, submission)
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

/// POST /rest/star
pub async fn star(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<StarParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    let user_id = &claims.sub;

    // 确定收藏类型和 ID
    let (item_type, item_id) = if let Some(artist_id) = params.artist_id {
        (StarItemType::Artist, artist_id)
    } else if let Some(album_id) = params.album_id {
        (StarItemType::Album, album_id)
    } else if let Some(song_id) = params.song_id {
        (StarItemType::Song, song_id)
    } else {
        return Err(AppError::missing_parameter(
            "id or artist_id/album_id/song_id",
        ));
    };

    // 调用 Service 层
    state
        .library_service
        .star_item(user_id, item_type, &item_id)
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

/// POST /rest/unstar
pub async fn unstar(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<StarParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    let user_id = &claims.sub;

    // 确定收藏类型和 ID
    let (item_type, item_id) = if let Some(artist_id) = params.artist_id {
        (StarItemType::Artist, artist_id)
    } else if let Some(album_id) = params.album_id {
        (StarItemType::Album, album_id)
    } else if let Some(song_id) = params.song_id {
        (StarItemType::Song, song_id)
    } else {
        return Err(AppError::missing_parameter(
            "id or artist_id/album_id/song_id",
        ));
    };

    // 调用 Service 层
    state
        .library_service
        .unstar_item(user_id, item_type, &item_id)
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

/// POST /rest/setRating
pub async fn set_rating(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<SetRatingParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    let user_id = &claims.sub;

    // 调用 Service 层 (包含验证逻辑)
    state
        .library_service
        .set_rating(user_id, &params.id, params.rating)
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
) -> Result<Json<SubsonicResponse<RatingResponse>>, AppError> {
    let user_id = &claims.sub;

    // 调用 Service 层
    let rating = state
        .library_service
        .get_rating(user_id, &params.id)
        .await?;

    let result = RatingResponse {
        id: params.id.clone(),
        rating: rating.unwrap_or(0),
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

/// GET /rest/getStarred
pub async fn get_starred(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    _params: Query<ScanParams>,
) -> Result<Json<SubsonicResponse<crate::models::response::StarredResponse>>, AppError> {
    let user_id = &claims.sub;

    // 调用 Service 层 (并行查询三个表)
    let starred_items = state.library_service.get_starred_items(user_id).await?;

    let result = crate::models::response::StarredResponse {
        artist: if starred_items.artists.is_empty() {
            None
        } else {
            Some(ArtistResponse::from_dtos(starred_items.artists))
        },
        album: if starred_items.albums.is_empty() {
            None
        } else {
            Some(AlbumResponse::from_dtos(starred_items.albums))
        },
        song: if starred_items.songs.is_empty() {
            None
        } else {
            Some(Song::from_dtos(starred_items.songs))
        },
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
    library_service: Arc<LibraryService>,
) -> Router {
    let scan_state = ScanState {
        scanning: Arc::new(Mutex::new(false)),
        current: Arc::new(Mutex::new(0)),
        count: Arc::new(Mutex::new(0)),
    };

    let library_state = LibraryState {
        pool: pool.clone(),
        scan_service,
        library_service,
        scan_state,
    };

    Router::new()
        .route("/rest/getScanStatus", get(get_scan_status))
        .route("/rest/startScan", get(start_scan))
        .route("/rest/scrobble", post(scrobble))
        .route("/rest/star", post(star))
        .route("/rest/unstar", post(unstar))
        .route("/rest/setRating", post(set_rating))
        .route("/rest/getRating", get(get_rating))
        .route("/rest/getStarred", get(get_starred))
        .with_state(library_state)
}
