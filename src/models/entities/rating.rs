//! 评分数据库实体
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::utils::id_builder;

/// 评分实体 (完整数据库表结构)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Rating {
    pub id: String,
    pub user_id: String,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub song_id: Option<String>,
    pub rating: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
            id: id_builder::generate_id(),
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
