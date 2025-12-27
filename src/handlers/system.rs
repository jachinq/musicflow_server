// handlers/system.rs
use axum::{Router, routing::get, Json};
use serde::Serialize;
use crate::models::response::{SubsonicResponse, ResponseContainer};

// GET /rest/ping
pub async fn ping() -> Json<SubsonicResponse<()>> {
    Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: None,
        },
    })
}

// GET /rest/getLicense
pub async fn get_license() -> Json<SubsonicResponse<LicenseResponse>> {
    Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: Some(LicenseResponse {
                valid: true,
                email: "admin@example.com".to_string(),
                key: "licensed".to_string(),
            }),
        },
    })
}

#[derive(Debug, Serialize)]
pub struct LicenseResponse {
    pub valid: bool,
    pub email: String,
    pub key: String,
}

pub fn routes() -> Router {
    Router::new()
        .route("/rest/ping", get(ping))
        .route("/rest/getLicense", get(get_license))
}