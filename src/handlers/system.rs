// handlers/system.rs
use axum::{routing::get, Router};
use serde::Serialize;

use crate::extractors::Format;
use crate::models::response::ToXml;
use crate::response::ApiResponse;

// GET /rest/ping
pub async fn ping(Format(format): Format) -> ApiResponse<()> {
    ApiResponse::ok(None, format)
}

// GET /rest/getLicense
pub async fn get_license(Format(format): Format) -> ApiResponse<LicenseResponse> {
    let license = LicenseResponse {
        valid: true,
        email: "admin@example.com".to_string(),
        key: "licensed".to_string(),
    };

    ApiResponse::ok(Some(license), format)
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseResponse {
    pub valid: bool,   // JSON: "valid" (干净!)
    pub email: String, // JSON: "email"
    pub key: String,   // JSON: "key"
}

// 实现 ToXml trait 用于 XML 序列化
impl ToXml for LicenseResponse {
    fn to_xml_element(&self) -> String {
        format!(
            "<license valid=\"{}\" email=\"{}\" key=\"{}\"/>",
            self.valid, self.email, self.key
        )
    }
}

pub fn routes() -> Router {
    Router::new()
        .route("/rest/ping", get(ping))
        .route("/rest/getLicense", get(get_license))
}
