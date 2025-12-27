//! 评分模型

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 评分实体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Rating {
    pub id: String,
    pub user_id: String,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub song_id: Option<String>,
    pub rating: i32,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

/// 评分响应（Subsonic 格式）
#[derive(Debug, Serialize, Deserialize)]
pub struct RatingResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@rating")]
    pub rating: i32,
}

/// 设置评分请求
#[derive(Debug, Deserialize)]
pub struct SetRatingRequest {
    pub id: String,
    pub rating: i32,
}

/// 获取评分请求
#[derive(Debug, Deserialize)]
pub struct GetRatingRequest {
    pub id: String,
}

impl Rating {
    pub fn new(
        user_id: String,
        artist_id: Option<String>,
        album_id: Option<String>,
        song_id: Option<String>,
        rating: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            artist_id,
            album_id,
            song_id,
            rating,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}