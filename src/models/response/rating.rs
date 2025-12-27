//! 评分响应模型 (Subsonic API 格式)

use serde::{Deserialize, Serialize};

/// 评分响应 (Subsonic 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingResponse {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@rating")]
    pub rating: i32,
}
