//! 播放队列响应模型 (Subsonic API 格式)
#![allow(dead_code)]

use super::{Song, ToXml};
use serde::{Deserialize, Serialize};

/// 播放队列响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayQueueResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    pub position: i64,
    pub username: String,
    pub changed: String,
    pub changed_by: String,
    #[serde(rename = "entry")]
    pub entries: Vec<Song>,
}

impl ToXml for PlayQueueResponse {
    fn to_xml_element(&self) -> String {
        let mut xml = String::new();

        xml.push_str("<playQueue");
        if let Some(current) = &self.current {
            xml.push_str(&format!(" current=\"{}\"", current));
        }
        xml.push_str(&format!(" position=\"{}\"", self.position));
        xml.push_str(&format!(" username=\"{}\"", self.username));
        xml.push_str(&format!(" changed=\"{}\"", self.changed));
        xml.push_str(&format!(" changedBy=\"{}\"", self.changed_by));
        xml.push_str(">");

        for song in &self.entries {
            xml.push_str(&song.to_xml_element());
        }

        xml.push_str("</playQueue>");
        xml
    }
}

/// PlayQueue 包装器 (用于顶层响应)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayQueueWrapper {
    pub play_queue: PlayQueueResponse,
}

impl ToXml for PlayQueueWrapper {
    fn to_xml_element(&self) -> String {
        self.play_queue.to_xml_element()
    }
}
