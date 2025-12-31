//! 用户管理端点处理器
#![allow(dead_code)]

use axum::{extract::Query, routing::get, Router};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::extractors::Format;
use crate::models::dto::{ChangePasswordRequest, CreateUserRequest, UpdateUserRequest};
use crate::models::response::{UserResponse, UsersResponse};
use crate::response::ApiResponse;
use crate::services::UserService;

/// 通用用户参数
#[derive(Debug, Deserialize)]
pub struct UserParams {
    pub username: Option<String>,
}

/// GET /rest/getUser - 获取用户信息
pub async fn get_user(
    axum::extract::State(user_service): axum::extract::State<Arc<UserService>>,
    Query(params): Query<UserParams>,
    Format(format): Format,
) -> Result<ApiResponse<UserResponse>, AppError> {
    let username = params
        .username
        .as_deref()
        .ok_or_else(|| AppError::missing_parameter("username"))?;

    // 调用 Service 层
    let user = user_service.get_user(username).await?;
    let user_response: UserResponse = user.into();

    Ok(ApiResponse::ok(Some(user_response), format))
}

/// GET /rest/getUsers - 获取所有用户(仅管理员)
pub async fn get_users(
    axum::extract::State(user_service): axum::extract::State<Arc<UserService>>,
    Format(format): Format,
) -> Result<ApiResponse<UsersResponse>, AppError> {
    // TODO: 需要从认证中间件获取当前用户名
    let current_user = "admin"; // 临时硬编码,后续需从认证中间件获取

    // 调用 Service 层 (包含权限检查)
    let users = user_service.get_all_users(current_user).await?;

    let user_responses: Vec<UserResponse> = users.into_iter().map(|u| u.into()).collect();

    let result = UsersResponse {
        users: user_responses,
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// POST /rest/createUser - 创建用户(仅管理员)
pub async fn create_user(
    axum::extract::State(user_service): axum::extract::State<Arc<UserService>>,
    Query(body): Query<CreateUserRequest>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    // TODO: 需要从认证中间件获取当前用户名
    let current_user = "admin"; // 临时硬编码

    // 调用 Service 层 (包含权限检查和用户存在性检查)
    user_service.create_user(current_user, body).await?;

    Ok(ApiResponse::ok(None, format))
}

/// POST /rest/deleteUser - 删除用户(仅管理员)
pub async fn delete_user(
    axum::extract::State(user_service): axum::extract::State<Arc<UserService>>,
    Query(params): Query<UserParams>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    let username = params
        .username
        .ok_or_else(|| AppError::missing_parameter("username"))?;

    // TODO: 需要从认证中间件获取当前用户名
    let current_user = "admin"; // 临时硬编码

    // 调用 Service 层 (包含权限检查和自删除检查)
    user_service.delete_user(current_user, &username).await?;

    Ok(ApiResponse::ok(None, format))
}

/// POST /rest/updateUser - 更新用户(仅管理员)
pub async fn update_user(
    axum::extract::State(user_service): axum::extract::State<Arc<UserService>>,
    Query(params): Query<UserParams>,
    Query(body): Query<UpdateUserRequest>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    let username = params
        .username
        .ok_or_else(|| AppError::missing_parameter("username"))?;

    // TODO: 需要从认证中间件获取当前用户名
    let current_user = "admin"; // 临时硬编码

    // 调用 Service 层 (包含权限检查和动态SQL构建)
    user_service
        .update_user(current_user, &username, body)
        .await?;

    Ok(ApiResponse::ok(None, format))
}

/// POST /rest/changePassword - 修改密码
pub async fn change_password(
    axum::extract::State(user_service): axum::extract::State<Arc<UserService>>,
    Query(body): Query<ChangePasswordRequest>,
    Format(format): Format,
) -> Result<ApiResponse<()>, AppError> {
    // TODO: 需要从认证中间件获取当前用户名
    let current_user = "admin"; // 临时硬编码

    // 调用 Service 层 (包含权限检查)
    user_service
        .change_password(current_user, &body.username, &body.password)
        .await?;

    Ok(ApiResponse::ok(None, format))
}

pub fn routes() -> Router<Arc<UserService>> {
    Router::new()
        .route("/rest/getUser", get(get_user))
        .route("/rest/getUsers", get(get_users))
        .route("/rest/createUser", get(create_user))
        .route("/rest/deleteUser", get(delete_user))
        .route("/rest/updateUser", get(update_user))
        .route("/rest/changePassword", get(change_password))
}
