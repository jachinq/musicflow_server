//! 认证中间件

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::models::user::User;
use crate::utils::verify_password;

/// 标准错误响应结构
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// 用于内部请求上下文传递的用户声明
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // user_id
    pub username: String,
    pub is_admin: bool,
    pub exp: usize,
    pub iat: usize,
}

/// 从请求扩展中提取用户认证信息
/// 由认证中间件设置，不再使用 JWT token
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 从请求扩展中获取 Claims（由认证中间件设置）
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "未找到认证信息，请提供有效的认证参数".to_string(),
            ))
    }
}

/// 尝试使用 Subsonic 查询参数进行认证
async fn try_subsonic_auth(
    query: &str,
    pool: &SqlitePool,
) -> Option<User> {
    // 解析查询参数
    let params: std::collections::HashMap<String, String> =
        serde_urlencoded::from_str(query).ok()?;

    // 提取用户名
    let username = params.get("u")?;

    // 尝试密码认证 (p 参数)
    if let Some(password) = params.get("p") {
        return authenticate_with_password(username, password, pool).await.ok();
    }

    // 尝试 token + salt 认证 (t + s 参数)
    if let (Some(token), Some(salt)) = (params.get("t"), params.get("s")) {
        return authenticate_subsonic(username, token, salt, pool).await.ok();
    }

    None
}

/// 通过密码直接认证
async fn authenticate_with_password(
    username: &str,
    password: &str,
    pool: &SqlitePool,
) -> Result<User, ()> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(|_| ())?
    .ok_or(())?;

    // 验证密码
    let is_valid = verify_password(password, &user.password_hash)
        .map_err(|_| ())?;

    if !is_valid {
        return Err(());
    }

    Ok(user)
}

/// Subsonic token 认证（MD5 方式）
async fn authenticate_subsonic(
    username: &str,
    token: &str,
    salt: &str,
    pool: &SqlitePool,
) -> Result<User, ()> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .map_err(|_| ())?
    .ok_or(())?;

    // 获取用户的 API 密码
    let api_password = user.api_password.as_ref().ok_or(())?;

    // 计算预期的 token: MD5(api_password + salt)
    let expected_token = crate::utils::generate_subsonic_token(api_password, salt);

    // 验证 token(不区分大小写)
    if !expected_token.eq_ignore_ascii_case(token) {
        return Err(());
    }

    Ok(user)
}

/// 认证中间件函数
pub async fn auth_middleware(
    mut req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    // 检查路径是否需要认证
    let path = req.uri().path();

    // 允许公开访问的端点
    if path.starts_with("/rest/ping")
        || path.starts_with("/rest/getLicense")
        || path.starts_with("/api/auth") {
        return next.run(req).await;
    }

    // 从扩展中获取数据库连接池
    let pool = match req.extensions()
        .get::<std::sync::Arc<SqlitePool>>()
        .cloned() {
        Some(pool) => pool,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "InternalServerError".to_string(),
                    message: "服务器配置错误".to_string(),
                    details: Some("数据库连接池未初始化".to_string()),
                }),
            ).into_response();
        }
    };

    // 尝试 Subsonic 查询参数认证
    if let Some(query) = req.uri().query() {
        if let Some(user) = try_subsonic_auth(query, &pool).await {
            // 创建 Claims 并添加到请求扩展中
            let claims = Claims {
                sub: user.id.clone(),
                username: user.username.clone(),
                is_admin: user.is_admin,
                exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
                iat: chrono::Utc::now().timestamp() as usize,
            };

            req.extensions_mut().insert(claims);
            return next.run(req).await;
        }
    }

    // 认证失败
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Unauthorized".to_string(),
            message: "认证失败".to_string(),
            details: Some("请提供有效的认证参数 (u+p 或 u+t+s)".to_string()),
        }),
    ).into_response()
}

/// 管理员权限检查中间件
pub async fn admin_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    // 从请求扩展中获取 claims
    let claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    message: "认证失败".to_string(),
                    details: Some("未找到认证信息，请先登录".to_string()),
                }),
            ).into_response();
        }
    };

    if !claims.is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Forbidden".to_string(),
                message: "权限不足".to_string(),
                details: Some("需要管理员权限才能访问此资源".to_string()),
            }),
        ).into_response();
    }

    next.run(req).await
}

/// 权限检查辅助函数：从 Claims 中获取用户权限
pub async fn get_user_permissions(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> Result<UserPermissions, StatusCode> {
    let permissions = sqlx::query_as::<_, UserPermissions>(
        "SELECT
            download_role,
            upload_role,
            playlist_role,
            cover_art_role,
            comment_role,
            podcast_role,
            share_role,
            video_conversion_role,
            scrobbling_enabled
         FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(permissions)
}

/// 用户权限结构体
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserPermissions {
    pub download_role: bool,
    pub upload_role: bool,
    pub playlist_role: bool,
    pub cover_art_role: bool,
    pub comment_role: bool,
    pub podcast_role: bool,
    pub share_role: bool,
    pub video_conversion_role: bool,
    pub scrobbling_enabled: bool,
}

impl UserPermissions {
    /// 检查下载权限
    pub fn can_download(&self) -> bool {
        self.download_role
    }

    /// 检查上传权限
    pub fn can_upload(&self) -> bool {
        self.upload_role
    }

    /// 检查播放列表权限
    pub fn can_manage_playlist(&self) -> bool {
        self.playlist_role
    }

    /// 检查封面艺术权限
    pub fn can_access_cover_art(&self) -> bool {
        self.cover_art_role
    }

    /// 检查评论权限
    pub fn can_comment(&self) -> bool {
        self.comment_role
    }

    /// 检查播客权限
    pub fn can_access_podcast(&self) -> bool {
        self.podcast_role
    }

    /// 检查分享权限
    pub fn can_share(&self) -> bool {
        self.share_role
    }

    /// 检查视频转换权限
    pub fn can_convert_video(&self) -> bool {
        self.video_conversion_role
    }

    /// 检查Scrobble权限
    pub fn can_scrobble(&self) -> bool {
        self.scrobbling_enabled
    }
}
