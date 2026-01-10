//! 库管理端点处理器
#![allow(dead_code)]

use crate::error::AppError;
use crate::extractors::Format;
use crate::models::response::{
    AlbumResponse, ArtistResponse, RatingResponse, RatingResponseWrapper, Song, Starred2Response,
    Starred2ResponseWrapper, StarredResponse, StarredResponseWrapper, ToXml,
};
use crate::response::ApiResponse;
use crate::services::{LibraryService, ScanService, StarItemType};
use axum::{extract::Query, routing::get, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 扫描状态
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatusResponse {
    pub scan_status: ScanStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanStatus {
    scanning: bool,
    count: usize,
    current: usize,
}

// ========== XML 序列化实现 ==========

impl ToXml for ScanStatusResponse {
    fn to_xml_element(&self) -> String {
        format!(
            r#"<scanStatus scanning="{}" count="{}" current="{}"/>"#,
            self.scan_status.scanning, self.scan_status.count, self.scan_status.current
        )
    }
}

/// 扫描请求参数
#[derive(Debug, Deserialize)]
pub struct ScanParams {
    // 认证参数已由中间件处理,这里不需要
}

/// 收藏参数
#[derive(Debug, Deserialize)]
pub struct StarParams {
    pub id: Option<String>,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub song_id: Option<String>,
}

/// 评分参数
#[derive(Debug, Deserialize)]
pub struct SetRatingParams {
    pub id: String,
    pub rating: i32,
}

/// 获取评分参数
#[derive(Debug, Deserialize)]
pub struct GetRatingParams {
    pub id: String,
}

/// Scrobble 参数
#[derive(Debug, Deserialize)]
pub struct ScrobbleParams {
    pub id: String,
    pub submission: Option<String>,
    pub time: Option<i64>,
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
    Format(format): Format,
) -> Result<ApiResponse<ScanStatusResponse>, AppError> {
    let scanning = *state.scan_state.scanning.lock().await;

    // 从 service 层获取歌曲总数
    let count = state.library_service.get_song_count().await?;

    let result = ScanStatusResponse {
        scan_status: ScanStatus {
            scanning,
            count,
            current: 0,
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// POST /rest/startScan
pub async fn start_scan(
    axum::extract::State(state): axum::extract::State<LibraryState>,
    _params: Query<ScanParams>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
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
    });

    Ok(ApiResponse::ok(None, format))
}

/// POST /rest/scrobble
pub async fn scrobble(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<ScrobbleParams>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
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
    let submission = params.submission.unwrap_or("True".to_string());
    let submission = submission == "True";

    // 调用 Service 层 (带事务保护)
    state
        .library_service
        .submit_scrobble(&claims.sub, &params.id, timestamp, submission)
        .await?;

    Ok(ApiResponse::ok(None, format))
}

/// POST /rest/star
pub async fn star(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<StarParams>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    let user_id = &claims.sub;

    // 确定收藏类型和 ID
    let (item_type, item_id) = if let Some(artist_id) = params.artist_id {
        (StarItemType::Artist, artist_id)
    } else if let Some(album_id) = params.album_id {
        (StarItemType::Album, album_id)
    } else if let Some(song_id) = params.song_id {
        (StarItemType::Song, song_id)
    } else if let Some(id) = params.id {
        // 默认作为歌曲ID处理
        (StarItemType::Song, id)
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

    Ok(ApiResponse::ok(None, format))
}

/// POST /rest/unstar
pub async fn unstar(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<StarParams>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    let user_id = &claims.sub;

    // 确定收藏类型和 ID
    let (item_type, item_id) = if let Some(artist_id) = params.artist_id {
        (StarItemType::Artist, artist_id)
    } else if let Some(album_id) = params.album_id {
        (StarItemType::Album, album_id)
    } else if let Some(song_id) = params.song_id {
        (StarItemType::Song, song_id)
    } else if let Some(id) = params.id {
        // 默认作为歌曲ID处理
        (StarItemType::Song, id)
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

    Ok(ApiResponse::ok(None, format))
}

/// POST /rest/setRating
pub async fn set_rating(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<SetRatingParams>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    let user_id = &claims.sub;

    // 调用 Service 层 (包含验证逻辑)
    state
        .library_service
        .set_rating(user_id, &params.id, params.rating)
        .await?;

    Ok(ApiResponse::ok(None, format))
}

/// GET /rest/getRating
pub async fn get_rating(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    Query(params): Query<GetRatingParams>,
    Format(format): Format,
) -> Result<ApiResponse<RatingResponseWrapper>, AppError> {
    let user_id = &claims.sub;

    tracing::info!("get_rating: user_id = {}, params = {:?}", user_id, params);

    // 调用 Service 层
    let rating = state
        .library_service
        .get_rating(user_id, &params.id)
        .await?;

    let result = RatingResponseWrapper {
        rating: RatingResponse {
            id: params.id.clone(),
            rating: rating.unwrap_or(0),
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getStarred
pub async fn get_starred(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    _params: Query<ScanParams>,
    Format(format): Format,
) -> Result<ApiResponse<StarredResponseWrapper>, AppError> {
    let user_id = &claims.sub;

    // 调用 Service 层 (并行查询三个表)
    let starred_items = state.library_service.get_starred_items(user_id).await?;

    let result = StarredResponseWrapper {
        starred: StarredResponse {
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
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getStarred2
pub async fn get_starred2(
    claims: crate::middleware::auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<LibraryState>,
    _params: Query<ScanParams>,
    Format(format): Format,
) -> Result<ApiResponse<Starred2ResponseWrapper>, AppError> {
    let user_id = &claims.sub;

    // 调用 Service 层 (并行查询三个表，返回详细信息)
    let starred_items = state
        .library_service
        .get_starred_items_with_details(user_id)
        .await?;
    // tracing::info!("starred_items: {:?}", starred_items);

    let result = Starred2ResponseWrapper {
        starred2: Starred2Response {
            artist: if starred_items.artists.is_empty() {
                None
            } else {
                Some(ArtistResponse::from_starred_dtos(starred_items.artists))
            },
            album: if starred_items.albums.is_empty() {
                None
            } else {
                Some(AlbumResponse::from_dto_details(starred_items.albums))
            },
            song: if starred_items.songs.is_empty() {
                None
            } else {
                Some(Song::from_complex_dtos(starred_items.songs))
            },
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
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
        .route("/rest/scrobble", get(scrobble))
        .route("/rest/star", get(star))
        .route("/rest/unstar", get(unstar))
        .route("/rest/setRating", get(set_rating))
        .route("/rest/getRating", get(get_rating))
        .route("/rest/getStarred", get(get_starred))
        .route("/rest/getStarred2", get(get_starred2))
        .with_state(library_state)
}
