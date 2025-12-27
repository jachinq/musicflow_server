//! 用户管理端点处理器
#![allow(dead_code)]

use axum::{
    Router,
    routing::{get, post},
    extract::Query,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::response::{SubsonicResponse, ResponseContainer, UserResponse, UsersResponse};
use crate::models::dto::{CreateUserRequest, UpdateUserRequest, ChangePasswordRequest};
use crate::models::entities::User;
use crate::services::AuthService;

/// 通用用户参数
#[derive(Debug, Deserialize)]
pub struct UserParams {
    pub username: Option<String>,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// GET /rest/getUser - 获取用户信息
pub async fn get_user(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<UserParams>,
) -> Result<Json<SubsonicResponse<UserResponse>>, AppError> {
    let username = params.username.as_deref().unwrap_or(&params.u);

    // 查询用户
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(&*pool)
    .await?;

    let user = user.ok_or_else(|| AppError::not_found("User"))?;

    let user_response: UserResponse = user.into();

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: Some(user_response),
        },
    }))
}

/// GET /rest/getUsers - 获取所有用户（仅管理员）
pub async fn get_users(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<UserParams>,
) -> Result<Json<SubsonicResponse<UsersResponse>>, AppError> {
    // 检查当前用户是否为管理员
    let is_admin = sqlx::query_scalar::<_, bool>(
        "SELECT is_admin FROM users WHERE username = ?"
    )
    .bind(&params.u)
    .fetch_optional(&*pool)
    .await?;

    if !is_admin.unwrap_or(false) {
        return Err(AppError::access_denied("Admin only"));
    }

    // 获取所有用户
    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users ORDER BY username"
    )
    .fetch_all(&*pool)
    .await?;

    let user_responses: Vec<UserResponse> = users
        .into_iter()
        .map(|u| u.into())
        .collect();

    let result = UsersResponse {
        user: user_responses,
    };

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: Some(result),
        },
    }))
}

/// POST /rest/createUser - 创建用户（仅管理员）
pub async fn create_user(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<UserParams>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 检查当前用户是否为管理员
    let is_admin = sqlx::query_scalar::<_, bool>(
        "SELECT is_admin FROM users WHERE username = ?"
    )
    .bind(&params.u)
    .fetch_optional(&*pool)
    .await?;

    if !is_admin.unwrap_or(false) {
        return Err(AppError::access_denied("Admin only"));
    }

    // 检查用户名是否已存在
    let existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)"
    )
    .bind(&body.username)
    .fetch_one(&*pool)
    .await?;

    if existing {
        return Err(AppError::validation_error("Username already exists"));
    }

    // 创建用户
    // 从环境变量获取JWT密钥，如果没有则使用默认值
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let auth_service = AuthService::new((*pool).clone(), jwt_secret);
    let _user = auth_service.register(body).await?;

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: None,
        },
    }))
}

/// POST /rest/deleteUser - 删除用户（仅管理员）
pub async fn delete_user(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<UserParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    let username = params.username.ok_or_else(|| AppError::missing_parameter("username"))?;

    // 检查当前用户是否为管理员
    let is_admin = sqlx::query_scalar::<_, bool>(
        "SELECT is_admin FROM users WHERE username = ?"
    )
    .bind(&params.u)
    .fetch_optional(&*pool)
    .await?;

    if !is_admin.unwrap_or(false) {
        return Err(AppError::access_denied("Admin only"));
    }

    // 不能删除自己
    if username == params.u {
        return Err(AppError::validation_error("Cannot delete yourself"));
    }

    // 删除用户
    sqlx::query("DELETE FROM users WHERE username = ?")
        .bind(&username)
        .execute(&*pool)
        .await?;

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: None,
        },
    }))
}

/// POST /rest/updateUser - 更新用户（仅管理员）
pub async fn update_user(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<UserParams>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    let username = params.username.ok_or_else(|| AppError::missing_parameter("username"))?;

    // 检查当前用户是否为管理员
    let is_admin = sqlx::query_scalar::<_, bool>(
        "SELECT is_admin FROM users WHERE username = ?"
    )
    .bind(&params.u)
    .fetch_optional(&*pool)
    .await?;

    if !is_admin.unwrap_or(false) {
        return Err(AppError::access_denied("Admin only"));
    }

    // 构建更新查询
    let mut query_parts = Vec::new();

    if let Some(_email) = &body.email {
        query_parts.push("email = ?");
    }

    if let Some(_max_bitrate) = body.max_bitrate {
        query_parts.push("max_bitrate = ?");
    }

    if let Some(_download_role) = body.download_role {
        query_parts.push("download_role = ?");
    }

    if let Some(_upload_role) = body.upload_role {
        query_parts.push("upload_role = ?");
    }

    if let Some(_playlist_role) = body.playlist_role {
        query_parts.push("playlist_role = ?");
    }

    if let Some(_cover_art_role) = body.cover_art_role {
        query_parts.push("cover_art_role = ?");
    }

    if let Some(_comment_role) = body.comment_role {
        query_parts.push("comment_role = ?");
    }

    if let Some(_podcast_role) = body.podcast_role {
        query_parts.push("podcast_role = ?");
    }

    if let Some(_share_role) = body.share_role {
        query_parts.push("share_role = ?");
    }

    if let Some(_video_conversion_role) = body.video_conversion_role {
        query_parts.push("video_conversion_role = ?");
    }

    if let Some(_scrobbling_enabled) = body.scrobbling_enabled {
        query_parts.push("scrobbling_enabled = ?");
    }

    if !query_parts.is_empty() {
        let query = format!(
            "UPDATE users SET {}, updated_at = CURRENT_TIMESTAMP WHERE username = ?",
            query_parts.join(", ")
        );
        let mut final_query = sqlx::query(&query);

        // 重新构建查询以绑定参数
        if let Some(email) = &body.email {
            final_query = final_query.bind(email);
        }
        if let Some(max_bitrate) = body.max_bitrate {
            final_query = final_query.bind(max_bitrate);
        }
        if let Some(download_role) = body.download_role {
            final_query = final_query.bind(download_role);
        }
        if let Some(upload_role) = body.upload_role {
            final_query = final_query.bind(upload_role);
        }
        if let Some(playlist_role) = body.playlist_role {
            final_query = final_query.bind(playlist_role);
        }
        if let Some(cover_art_role) = body.cover_art_role {
            final_query = final_query.bind(cover_art_role);
        }
        if let Some(comment_role) = body.comment_role {
            final_query = final_query.bind(comment_role);
        }
        if let Some(podcast_role) = body.podcast_role {
            final_query = final_query.bind(podcast_role);
        }
        if let Some(share_role) = body.share_role {
            final_query = final_query.bind(share_role);
        }
        if let Some(video_conversion_role) = body.video_conversion_role {
            final_query = final_query.bind(video_conversion_role);
        }
        if let Some(scrobbling_enabled) = body.scrobbling_enabled {
            final_query = final_query.bind(scrobbling_enabled);
        }

        final_query = final_query.bind(&username);
        final_query.execute(&*pool).await?;
    }

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: None,
        },
    }))
}

/// POST /rest/changePassword - 修改密码
pub async fn change_password(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<UserParams>,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 检查权限：用户只能修改自己的密码，或者管理员可以修改任意用户密码
    let is_admin = sqlx::query_scalar::<_, bool>(
        "SELECT is_admin FROM users WHERE username = ?"
    )
    .bind(&params.u)
    .fetch_optional(&*pool)
    .await?;

    let can_change = if is_admin.unwrap_or(false) {
        // 管理员可以修改任意用户
        true
    } else if params.u == body.username {
        // 用户可以修改自己的密码
        true
    } else {
        false
    };

    if !can_change {
        return Err(AppError::access_denied("Cannot change other user's password"));
    }

    // 使用 AuthService 来安全地修改密码
    // 从环境变量获取JWT密钥，如果没有则使用默认值
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let auth_service = AuthService::new((*pool).clone(), jwt_secret);

    // 需要先获取用户ID
    let user_id = sqlx::query_scalar::<_, String>(
        "SELECT id FROM users WHERE username = ?"
    )
    .bind(&body.username)
    .fetch_optional(&*pool)
    .await?
    .ok_or_else(|| AppError::not_found("User"))?;

    auth_service.change_password(&user_id, &body.password).await?;

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: None,
        },
    }))
}

pub fn routes() -> Router<Arc<SqlitePool>> {
    Router::new()
        .route("/rest/getUser", get(get_user))
        .route("/rest/getUsers", get(get_users))
        .route("/rest/createUser", post(create_user))
        .route("/rest/deleteUser", post(delete_user))
        .route("/rest/updateUser", post(update_user))
        .route("/rest/changePassword", post(change_password))
}
