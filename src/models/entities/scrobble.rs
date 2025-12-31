//! 播放记录数据库实体
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::utils::id_builder;

/// 播放记录实体 (完整数据库表结构)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Scrobble {
    pub id: String,
    pub user_id: String,
    pub song_id: String,
    pub timestamp: DateTime<Utc>,
    pub submission: bool,
    pub created_at: DateTime<Utc>,
}

impl Scrobble {
    pub fn new(
        user_id: String,
        song_id: String,
        timestamp: DateTime<Utc>,
        submission: bool,
    ) -> Self {
        Self {
            id: id_builder::generate_id(),
            user_id,
            song_id,
            timestamp,
            submission,
            created_at: Utc::now(),
        }
    }
}
