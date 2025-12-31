//! 播放列表数据库实体
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::utils::id_builder;

/// 播放列表实体 (完整数据库表结构)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Playlist {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub comment: Option<String>,
    pub is_public: bool,
    pub song_count: i32,
    pub duration: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Playlist {
    pub fn new(owner_id: String, name: String, comment: Option<String>, is_public: bool) -> Self {
        Self {
            id: id_builder::generate_id(),
            owner_id,
            name,
            comment,
            is_public,
            song_count: 0,
            duration: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
