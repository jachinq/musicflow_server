//! 收藏数据库实体
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

use crate::utils::id_builder;

/// 收藏实体 (完整数据库表结构)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Starred {
    pub id: String,
    pub user_id: String,
    pub artist_id: Option<String>,
    pub album_id: Option<String>,
    pub song_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Starred {
    pub fn new(user_id: String, artist_id: Option<String>, album_id: Option<String>, song_id: Option<String>) -> Self {
        Self {
            id: id_builder::generate_id(),
            user_id,
            artist_id,
            album_id,
            song_id,
            created_at: Utc::now(),
        }
    }
}
