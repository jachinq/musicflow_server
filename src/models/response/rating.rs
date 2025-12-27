//! 评分响应模型 (Subsonic API 格式)

use serde::{Deserialize, Serialize};

/// 评分响应 (Subsonic 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingResponse {
    pub id: String,
    pub rating: i32,
}
