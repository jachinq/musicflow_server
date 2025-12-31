//! 用户数据库实体
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::utils::id_builder;

/// 用户实体 (完整数据库表结构)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String, // 明文密码,用于 MD5 token 验证
    pub email: String,
    pub is_admin: bool,
    pub max_bitrate: i32,
    pub download_role: bool,
    pub upload_role: bool,
    pub playlist_role: bool,
    pub cover_art_role: bool,
    pub comment_role: bool,
    pub podcast_role: bool,
    pub share_role: bool,
    pub video_conversion_role: bool,
    pub scrobbling_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// 创建新用户（用于数据库插入）
    pub fn new(username: String, password: String, email: String, is_admin: bool) -> Self {
        Self {
            id: id_builder::generate_id(),
            username,
            password,
            email,
            is_admin,
            max_bitrate: 320,
            download_role: true,
            upload_role: false,
            playlist_role: true,
            cover_art_role: true,
            comment_role: false,
            podcast_role: false,
            share_role: true,
            video_conversion_role: false,
            scrobbling_enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
