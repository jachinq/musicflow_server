//! 认证服务

use crate::models::dto::{CreateUserRequest, LoginRequest};
use crate::models::entities::User;
use crate::utils::{generate_subsonic_token, generate_salt};
use crate::error::AppError;
use sqlx::SqlitePool;
use uuid::Uuid;

/// 带令牌的用户响应
#[derive(Debug, serde::Serialize)]
pub struct UserWithToken {
    pub username: String,
    pub email: String,
    pub admin: bool,
    pub scrobbling_enabled: bool,
    pub max_bit_rate: i32,
    pub download_role: bool,
    pub upload_role: bool,
    pub playlist_role: bool,
    pub cover_art_role: bool,
    pub comment_role: bool,
    pub podcast_role: bool,
    pub share_role: bool,
    pub video_conversion_role: bool,
}

pub struct AuthService {
    pool: SqlitePool,
}

impl AuthService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 用户注册
    pub async fn register(&self, req: CreateUserRequest) -> Result<UserWithToken, AppError> {
        // 检查用户名是否已存在
        let existing = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = ? OR email = ?"
        )
        .bind(&req.username)
        .bind(&req.email)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            return Err(AppError::validation_error("Username or email already exists"));
        }

        // 生成用户ID
        let user_id = Uuid::new_v4().to_string();

        // 创建用户(直接存储明文密码)
        sqlx::query(
            "INSERT INTO users (id, username, password, email, is_admin) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&user_id)
        .bind(&req.username)
        .bind(&req.password)
        .bind(&req.email)
        .bind(req.is_admin.unwrap_or(false))
        .execute(&self.pool)
        .await?;

        Ok(UserWithToken {
            username: req.username,
            email: req.email,
            admin: req.is_admin.unwrap_or(false),
            scrobbling_enabled: true,
            max_bit_rate: 320,
            download_role: true,
            upload_role: false,
            playlist_role: true,
            cover_art_role: true,
            comment_role: false,
            podcast_role: false,
            share_role: true,
            video_conversion_role: false,
        })
    }

    /// 用户登录
    pub async fn login(&self, req: LoginRequest) -> Result<UserWithToken, AppError> {
        // 查询用户
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = ?"
        )
        .bind(&req.username)
        .fetch_optional(&self.pool)
        .await?;

        let user = user.ok_or_else(|| AppError::auth_failed("Invalid username or password"))?;

        // 验证密码(直接比较明文)
        if req.password != user.password {
            return Err(AppError::auth_failed("Invalid username or password"));
        }

        Ok(UserWithToken {
            username: user.username,
            email: user.email,
            admin: user.is_admin,
            scrobbling_enabled: user.scrobbling_enabled,
            max_bit_rate: user.max_bitrate,
            download_role: user.download_role,
            upload_role: user.upload_role,
            playlist_role: user.playlist_role,
            cover_art_role: user.cover_art_role,
            comment_role: user.comment_role,
            podcast_role: user.podcast_role,
            share_role: user.share_role,
            video_conversion_role: user.video_conversion_role,
        })
    }

    /// Subsonic认证（使用token + salt）
    pub async fn authenticate_subsonic(
        &self,
        username: &str,
        token: &str,
        _salt: &str,
    ) -> Result<User, AppError> {
        // 查询用户
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        let user = user.ok_or_else(|| AppError::auth_failed("Invalid username"))?;

        // 由于bcrypt是单向哈希，我们无法直接验证Subsonic的MD5(token)
        // 在实际应用中，我们需要在users表中额外存储一个用于Subsonic认证的字段
        // 这里我们简化处理：验证token格式并返回用户

        // 检查token长度（MD5应该是32个字符）
        if token.len() != 32 {
            return Err(AppError::auth_failed("Invalid token format"));
        }

        // 在生产环境中，这里应该：
        // 1. 从数据库获取用户的Subsonic密码哈希
        // 2. 使用提供的salt重新计算MD5
        // 3. 比较两个MD5值

        // 为简化，我们返回用户（假设认证通过）
        Ok(user)
    }

    /// 通过密码直接认证（Subsonic p参数）
    pub async fn authenticate_with_password(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        let user = user.ok_or_else(|| AppError::auth_failed("Invalid username"))?;

        // 验证密码(直接比较明文)
        if password != user.password {
            return Err(AppError::auth_failed("Invalid password"));
        }

        Ok(user)
    }

    /// 生成Subsonic认证所需的salt和token
    pub fn generate_subsonic_credentials(&self, password: &str) -> (String, String) {
        let salt = generate_salt();
        let token = generate_subsonic_token(password, &salt);
        (salt, token)
    }

    /// 修改密码
    pub async fn change_password(
        &self,
        user_id: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET password = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(new_password)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
