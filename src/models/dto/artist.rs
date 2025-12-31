//! 艺术家数据传输对象

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 艺术家基础信息 DTO (用于搜索等简单查询)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ArtistDto {
    pub id: String,
    pub name: String,
}

/// 艺术家详细信息 DTO (包含统计信息)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ArtistDetailDto {
    pub id: String,
    pub name: String,
    pub cover_art_path: Option<String>,
}
