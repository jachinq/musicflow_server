//! 用户响应模型 (Subsonic API 格式)

use serde::{Deserialize, Serialize};
use crate::models::dto::UserDto;
use crate::models::entities::User;

/// 用户响应 (Subsonic 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersResponse {
    pub user: Vec<UserResponse>,
}

// DTO -> Response 转换
impl From<UserDto> for UserResponse {
    fn from(dto: UserDto) -> Self {
        Self {
            username: dto.username,
            email: dto.email,
            admin: dto.is_admin,
            scrobbling_enabled: dto.scrobbling_enabled,
            max_bit_rate: dto.max_bitrate,
            download_role: dto.download_role,
            upload_role: dto.upload_role,
            playlist_role: dto.playlist_role,
            cover_art_role: dto.cover_art_role,
            comment_role: dto.comment_role,
            podcast_role: dto.podcast_role,
            share_role: dto.share_role,
            video_conversion_role: dto.video_conversion_role,
        }
    }
}

// Entity -> Response 转换
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
