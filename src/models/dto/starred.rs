//! 收藏数据传输对象
#![allow(dead_code)]

use serde::Deserialize;

/// 收藏请求
#[derive(Debug, Deserialize)]
pub struct StarRequest {
    pub id: Option<String>,
    pub album_id: Option<String>,
    pub artist_id: Option<String>,
}
