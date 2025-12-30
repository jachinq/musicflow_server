//! 评分响应模型 (Subsonic API 格式)

use serde::{Deserialize, Serialize};
use super::ToXml;

/// 评分响应 (Subsonic 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingResponse {
    pub id: String,
    pub rating: i32,
}

// ========== XML 序列化实现 ==========

impl ToXml for RatingResponse {
    fn to_xml_element(&self) -> String {
        format!(
            r#"<rating id="{}" rating="{}"/>"#,
            self.id, self.rating
        )
    }
}
