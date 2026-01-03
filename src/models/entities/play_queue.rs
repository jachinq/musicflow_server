//! 播放队列数据库实体
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::utils::id_builder;

/// 播放队列实体 (完整数据库表结构)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayQueue {
    pub id: String,
    pub user_id: String,
    pub current_song_id: Option<String>,
    pub position: i64,
    pub changed_at: DateTime<Utc>,
    pub changed_by: String,
    pub updated_at: DateTime<Utc>,
}

impl PlayQueue {
    pub fn new(user_id: String, changed_by: String) -> Self {
        let now = Utc::now();
        Self {
            id: id_builder::generate_id(),
            user_id,
            current_song_id: None,
            position: 0,
            changed_at: now,
            changed_by,
            updated_at: now,
        }
    }
}

/// 播放队列歌曲关联实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayQueueSong {
    pub id: String,
    pub play_queue_id: String,
    pub song_id: String,
    pub song_order: i32,
}

impl PlayQueueSong {
    pub fn new(play_queue_id: String, song_id: String, song_order: i32) -> Self {
        Self {
            id: id_builder::generate_id(),
            play_queue_id,
            song_id,
            song_order,
        }
    }
}
