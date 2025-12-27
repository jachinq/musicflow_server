//! 播放记录模型
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 播放记录实体
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Scrobble {
    pub id: String,
    pub user_id: String,
    pub song_id: String,
    pub timestamp: DateTime<Utc>,
    pub submission: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

/// 播放记录请求
#[derive(Debug, Deserialize)]
pub struct ScrobbleRequest {
    pub id: String,
    pub submission: Option<bool>,
    pub time: Option<i64>,
}

impl Scrobble {
    pub fn new(user_id: String, song_id: String, timestamp: DateTime<Utc>, submission: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            song_id,
            timestamp,
            submission,
            created_at: Utc::now(),
        }
    }
}