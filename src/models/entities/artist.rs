//! 艺术家数据库实体

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 艺术家实体 - 对应 artists 表的完整结构
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub music_brainz_id: Option<String>,
    pub cover_art_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Artist {
    pub fn new(name: String, music_brainz_id: Option<String>, cover_art_path: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            music_brainz_id,
            cover_art_path,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
