//! 播放列表数据传输对象

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 播放列表 DTO (查询结果 - 含所有者)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlaylistDto {
    pub id: String,
    pub name: String,
    pub owner: String, // 所有者用户名
    pub is_public: bool,
    pub song_count: i32,
    pub duration: i32,
}

/// 创建播放列表请求
#[derive(Debug, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub song_id: Option<Vec<String>>,
}

/// 更新播放列表请求
#[derive(Debug, Deserialize)]
pub struct UpdatePlaylistRequest {
    pub name: Option<String>,
    pub comment: Option<String>,
    pub public: Option<bool>,
    pub song_id_to_add: Option<Vec<String>>,
    pub song_index_to_remove: Option<Vec<i32>>,
}
