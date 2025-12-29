//! 简单的 XML/JSON 序列化测试
//! 不依赖完整的 musicflow_server,只测试核心序列化功能

use serde::{Deserialize, Serialize};

/// Subsonic 响应容器
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "subsonic-response")]
pub struct SubsonicResponse<T> {
    #[serde(rename = "@xmlns", skip_serializing_if = "Option::is_none")]
    pub xmlns: Option<String>,

    #[serde(rename = "@status")]
    pub status: String,

    #[serde(rename = "@version")]
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SubsonicError>,

    #[serde(flatten)]
    pub data: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsonicError {
    #[serde(rename = "@code")]
    pub code: i32,

    #[serde(rename = "@message")]
    pub message: String,
}

impl<T> SubsonicResponse<T> {
    pub fn ok(data: Option<T>) -> Self {
        Self {
            xmlns: None,
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data,
        }
    }

    pub fn ok_xml(data: Option<T>) -> Self {
        Self {
            xmlns: Some("http://subsonic.org/restapi".to_string()),
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data,
        }
    }
}

impl<T: Serialize + Clone> SubsonicResponse<T> {
    pub fn to_xml(&self) -> Result<String, quick_xml::DeError> {
        let mut response = self.clone();
        if response.xmlns.is_none() {
            response.xmlns = Some("http://subsonic.org/restapi".to_string());
        }

        let xml = quick_xml::se::to_string(&response)?;
        Ok(format!(r#"<?xml version="1.0" encoding="UTF-8"?>{}"#, xml))
    }
}

#[derive(Debug, Clone, Serialize)]
struct License {
    #[serde(rename = "@valid")]
    valid: bool,
    #[serde(rename = "@email")]
    email: String,
    #[serde(rename = "@key")]
    key: String,
}

fn main() {
    println!("=== Subsonic API 格式测试 ===\n");

    // 测试 1: 空响应 JSON
    println!("【测试 1】空响应 - JSON 格式:");
    println!("{}", "=".repeat(60));
    let response_json: SubsonicResponse<()> = SubsonicResponse::ok(None);
    let json = serde_json::to_string_pretty(&response_json).unwrap();
    println!("{}\n", json);

    // 测试 2: 空响应 XML
    println!("【测试 2】空响应 - XML 格式:");
    println!("{}", "=".repeat(60));
    let response_xml: SubsonicResponse<()> = SubsonicResponse::ok_xml(None);
    let xml = response_xml.to_xml().unwrap();
    println!("{}\n", xml);

    // 测试 3: 带数据响应 JSON
    println!("【测试 3】带数据响应 - JSON 格式:");
    println!("{}", "=".repeat(60));
    let license = License {
        valid: true,
        email: "admin@example.com".to_string(),
        key: "ABC123".to_string(),
    };
    let response_with_data_json = SubsonicResponse::ok(Some(license.clone()));
    let json_with_data = serde_json::to_string_pretty(&response_with_data_json).unwrap();
    println!("{}\n", json_with_data);

    // 测试 4: 带数据响应 XML
    println!("【测试 4】带数据响应 - XML 格式:");
    println!("{}", "=".repeat(60));
    let response_with_data_xml = SubsonicResponse::ok_xml(Some(license));
    let xml_with_data = response_with_data_xml.to_xml().unwrap();
    println!("{}\n", xml_with_data);

    println!("✅ 所有测试完成!");
}
