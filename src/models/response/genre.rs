//! 流派响应结构
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// 单个流派
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genre {
    #[serde(rename = "$value")]
    pub name: String,
    pub song_count: i32,
    pub album_count: i32,
}

/// 流派列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenresResponse {
    pub genres: Genres,
}

/// 流派列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genres {
    #[serde(rename = "genre")]
    pub genres: Vec<Genre>,
}
