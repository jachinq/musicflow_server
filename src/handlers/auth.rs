//! 认证相关处理器

use axum::{
    Router,
    routing::{post, get},
    Json,
    extract::Query,
};
use serde::Deserialize;
use crate::services::{AuthService, UserWithToken};
use crate::models::user::{CreateUserRequest, LoginRequest};
use crate::error::AppError;
use std::sync::Arc;

/// 注册请求参数
#[derive(Debug, Deserialize)]
pub struct RegisterParams {
    pub username: String,
    pub password: String,
    pub email: String,
    pub is_admin: Option<bool>,
}

/// 登录请求参数
#[derive(Debug, Deserialize)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
}

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
    Json(params): Json<RegisterParams>,
) -> Result<Json<UserWithToken>, AppError> {
    let req = CreateUserRequest {
        username: params.username,
        password: params.password,
        email: params.email,
        is_admin: params.is_admin,
    };

    let user = auth_service.register(req).await?;
    Ok(Json(user))
}

/// 用户登录
pub async fn login(
    axum::extract::State(auth_service): axum::extract::State<Arc<AuthService>>,
    Json(params): Json<LoginParams>,
) -> Result<Json<UserWithToken>, AppError> {
    let req = LoginRequest {
        username: params.username,
        password: params.password,
    };

    let user = auth_service.login(req).await?;
    Ok(Json(user))
}

/// Subsonic 认证验证
pub async fn verify_subsonic_auth(
    axum::extract::State(auth_service): axum::extract::State<Arc<AuthService>>,
    Query(params): Query<SubsonicAuthParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 验证认证
    if let Some(token) = params.t {
        if let Some(salt) = params.s {
            // 使用 token + salt 认证
            auth_service.authenticate_subsonic(&params.u, &token, &salt).await?;
        } else {
            return Err(AppError::auth_failed("Missing salt parameter"));
        }
    } else if let Some(password) = params.p {
        // 使用密码明文认证
        auth_service.authenticate_with_password(&params.u, &password).await?;
    } else {
        return Err(AppError::auth_failed("Missing authentication parameters (t+s or p)"));
    }

    // 返回成功响应
    Ok(Json(serde_json::json!({
        "status": "ok",
        "version": "1.16.1"
    })))
}

/// 生成 Subsonic 认证凭据
pub async fn generate_subsonic_credentials(
    axum::extract::State(auth_service): axum::extract::State<Arc<AuthService>>,
    Json(params): Json<serde_json::Map<String, serde_json::Value>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let password = params.get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::validation_error("Password is required"))?;

    let (salt, token) = auth_service.generate_subsonic_credentials(password);

    Ok(Json(serde_json::json!({
        "salt": salt,
        "token": token
    })))
}

pub fn routes() -> Router<Arc<AuthService>> {
    Router::new()
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
       
        .route("/rest/verifySubsonicAuth", get(verify_subsonic_auth))
        .route("/api/auth/subsonic-credentials", post(generate_subsonic_credentials))
}
