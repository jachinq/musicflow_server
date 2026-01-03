//! 播放队列端点处理器

use axum::{routing::get, Router};
use axum_extra::extract::Query;
use serde::Deserialize;
use std::sync::Arc;

use crate::extractors::Format;
use crate::models::response::{PlayQueueResponse, PlayQueueWrapper, Song};
use crate::response::ApiResponse;
use crate::services::PlayQueueService;
use crate::{error::AppError, middleware::auth_middleware};

/// 播放队列处理器状态
#[derive(Clone)]
pub struct PlayQueueState {
    pub play_queue_service: Arc<PlayQueueService>,
}

/// 保存播放队列参数
#[derive(Debug, Deserialize)]
pub struct SavePlayQueueParams {
    pub id: Vec<String>,         // 歌曲 ID 列表
    pub current: Option<String>, // 当前播放歌曲 ID
    pub position: Option<i64>,   // 当前播放位置（毫秒）
    pub c: String,               // 客户端名称（用于 changed_by）
}

/// GET /rest/getPlayQueue - 获取用户的播放队列
pub async fn get_play_queue(
    claims: auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<PlayQueueState>,
    Format(format): Format,
) -> Result<ApiResponse<PlayQueueWrapper>, AppError> {
    let user_id = &claims.sub;

    // 调用 Service 层
    let play_queue = state.play_queue_service.get_play_queue(user_id).await?;

    match play_queue {
        Some(queue) => {
            // 转换为响应格式
            let songs: Vec<Song> = queue.songs.into_iter().map(|s| Song::from(s)).collect();

            let response = PlayQueueResponse {
                current: queue.current_song_id,
                position: queue.position,
                username: queue.username,
                changed: queue.changed_at,
                changed_by: queue.changed_by,
                entries: songs,
            };

            Ok(ApiResponse::ok(
                Some(PlayQueueWrapper {
                    play_queue: response,
                }),
                format,
            ))
        }
        None => {
            // 返回空的播放队列
            let response = PlayQueueResponse {
                current: None,
                position: 0,
                username: claims.username,
                changed: chrono::Utc::now().to_rfc3339(),
                changed_by: "system".to_string(),
                entries: vec![],
            };

            Ok(ApiResponse::ok(
                Some(PlayQueueWrapper {
                    play_queue: response,
                }),
                format,
            ))
        }
    }
}

/// GET/POST /rest/savePlayQueue - 保存用户的播放队列
pub async fn save_play_queue(
    claims: auth_middleware::Claims,
    axum::extract::State(state): axum::extract::State<PlayQueueState>,
    Query(params): Query<SavePlayQueueParams>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    let user_id = &claims.sub;

    // 验证歌曲 ID 列表不为空
    if params.id.is_empty() {
        return Err(AppError::MissingParameter("id".to_string()));
    }

    // 调用 Service 层保存播放队列
    state
        .play_queue_service
        .save_play_queue(
            user_id,
            params.id,
            params.current,
            params.position,
            &params.c,
        )
        .await?;

    // 返回空响应（status="ok"）
    Ok(ApiResponse::ok(None, format))
}

/// 注册播放队列路由
pub fn routes(state: PlayQueueState) -> Router {
    Router::new()
        .route("/rest/getPlayQueue", get(get_play_queue))
        .route(
            "/rest/savePlayQueue",
            get(save_play_queue).post(save_play_queue),
        )
        .with_state(state)
}
