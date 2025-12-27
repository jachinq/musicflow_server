//! 用户模型

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 用户实体（数据库）
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    #[serde(skip_serializing)]
    pub api_password: Option<String>,  // Subsonic API 密码(明文,用于 MD5 token 验证)
    pub email: String,
    #[serde(rename = "admin")]
    pub is_admin: bool,
    #[serde(rename = "maxBitRate")]
    pub max_bitrate: i32,
    #[serde(rename = "downloadRole")]
    pub download_role: bool,
    #[serde(rename = "uploadRole")]
    pub upload_role: bool,
    #[serde(rename = "playlistRole")]
    pub playlist_role: bool,
    #[serde(rename = "coverArtRole")]
    pub cover_art_role: bool,
    #[serde(rename = "commentRole")]
    pub comment_role: bool,
    #[serde(rename = "podcastRole")]
    pub podcast_role: bool,
    #[serde(rename = "shareRole")]
    pub share_role: bool,
    #[serde(rename = "videoConversionRole")]
    pub video_conversion_role: bool,
    #[serde(rename = "scrobblingEnabled")]
    pub scrobbling_enabled: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

/// 创建用户请求
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    pub is_admin: Option<bool>,
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 更新用户请求
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub max_bitrate: Option<i32>,
    pub download_role: Option<bool>,
    pub upload_role: Option<bool>,
    pub playlist_role: Option<bool>,
    pub cover_art_role: Option<bool>,
    pub comment_role: Option<bool>,
    pub podcast_role: Option<bool>,
    pub share_role: Option<bool>,
    pub video_conversion_role: Option<bool>,
    pub scrobbling_enabled: Option<bool>,
}

/// 用户响应（Subsonic 格式）
#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub username: String,
    pub email: String,
    pub admin: bool,
    pub scrobbling_enabled: bool,
    #[serde(rename = "maxBitRate")]
    pub max_bit_rate: i32,
    #[serde(rename = "downloadRole")]
    pub download_role: bool,
    #[serde(rename = "uploadRole")]
    pub upload_role: bool,
    #[serde(rename = "playlistRole")]
    pub playlist_role: bool,
    #[serde(rename = "coverArtRole")]
    pub cover_art_role: bool,
    #[serde(rename = "commentRole")]
    pub comment_role: bool,
    #[serde(rename = "podcastRole")]
    pub podcast_role: bool,
    #[serde(rename = "shareRole")]
    pub share_role: bool,
    #[serde(rename = "videoConversionRole")]
    pub video_conversion_role: bool,
}

/// 用户列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct UsersResponse {
    pub user: Vec<UserResponse>,
}

/// 修改密码请求
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub username: String,
    pub password: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            username: user.username,
            email: user.email,
            admin: user.is_admin,
            scrobbling_enabled: user.scrobbling_enabled,
            max_bit_rate: user.max_bitrate,
            download_role: user.download_role,
            upload_role: user.upload_role,
            playlist_role: user.playlist_role,
            cover_art_role: user.cover_art_role,
            comment_role: user.comment_role,
            podcast_role: user.podcast_role,
            share_role: user.share_role,
            video_conversion_role: user.video_conversion_role,
        }
    }
}

impl User {
    /// 创建新用户（用于数据库插入）
    pub fn new(
        username: String,
        password_hash: String,
        email: String,
        is_admin: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            username: username.clone(),
            password_hash,
            api_password: Some(username),  // 默认使用用户名作为 API 密码
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