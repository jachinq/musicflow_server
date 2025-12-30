//! 认证相关处理器，用于服务器后台的认证和授权
#![allow(dead_code)]

use axum::{
    Router,
    routing::post,
    Json,
};
use serde::Deserialize;
use crate::services::{AuthService, UserWithToken};
use crate::models::dto::{CreateUserRequest, LoginRequest};
use crate::error::AppError;
use std::sync::Arc;

/// Subsonic 认证参数
#[derive(Debug, Deserialize)]
pub struct SubsonicAuthParams {
    pub u: String,      // username
    pub t: Option<String>, // token
    pub s: Option<String>, // salt
    pub p: Option<String>, // password (明文)
    pub v: String,      // version
    pub c: String,      // client name
}

/// 用户注册
pub async fn register(
    axum::extract::State(auth_service): axum::extract::State<Arc<AuthService>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserWithToken>, AppError> {
    let user = auth_service.register(req).await?;
    Ok(Json(user))
}

/// 用户登录
pub async fn login(
    axum::extract::State(auth_service): axum::extract::State<Arc<AuthService>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<UserWithToken>, AppError> {
    let user = auth_service.login(req).await?;
    Ok(Json(user))
}

pub fn routes() -> Router<Arc<AuthService>> {
    Router::new()
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
}
