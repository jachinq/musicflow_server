//! 搜索结果响应模型

use serde::{Deserialize, Serialize};

/// 搜索结果响应包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultResponse {
    pub search_result: SearchResult,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(rename = "artist")]
    pub artist: Vec<super::ArtistResponse>,
    #[serde(rename = "album")]
    pub album: Vec<super::AlbumResponse>,
    #[serde(rename = "song")]
    pub song: Vec<super::SongResponse>,
}

/// 搜索结果2响应包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult2Response {
    pub search_result2: SearchResult2,
}

/// 搜索结果2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult2 {
    #[serde(rename = "artist")]
    pub artist: Vec<super::ArtistResponse>,
    #[serde(rename = "album")]
    pub album: Vec<super::AlbumResponse>,
    #[serde(rename = "song")]
    pub song: Vec<super::SongResponse>,
}

/// 搜索结果3响应包装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult3Response {
    pub search_result3: SearchResult3,
}

/// 搜索结果3 (包含更多详情)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult3 {
    #[serde(rename = "artist")]
    pub artist: Vec<super::ArtistResponse>,
    #[serde(rename = "album")]
    pub album: Vec<super::AlbumResponse>,
    #[serde(rename = "song")]
    pub song: Vec<super::SongResponse>,
}
