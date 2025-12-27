//! 收藏响应模型 (Subsonic API 格式)

use serde::{Deserialize, Serialize};
use super::{ArtistResponse, AlbumResponse, SongResponse};

/// 收藏响应 (Subsonic 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarredResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<Vec<ArtistResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<Vec<AlbumResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub song: Option<Vec<SongResponse>>,
}
