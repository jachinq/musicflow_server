//! 流派响应结构
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use super::ToXml;

/// 单个流派
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genre {
    pub value: String,
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

// ========== XML 序列化实现 ==========

impl ToXml for Genre {
    fn to_xml_element(&self) -> String {
        format!(
            r#"<genre songCount="{}" albumCount="{}">{}</genre>"#,
            self.song_count, self.album_count, self.value
        )
    }
}

impl ToXml for GenresResponse {
    fn to_xml_element(&self) -> String {
        self.genres.to_xml_element()
    }
}

impl ToXml for Genres {
    fn to_xml_element(&self) -> String {
        let mut xml = String::from("<genres>");
        for genre in &self.genres {
            xml.push_str(&genre.to_xml_element());
        }
        xml.push_str("</genres>");
        xml
    }
}
