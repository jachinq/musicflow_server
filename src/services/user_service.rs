//! 用户管理服务
//!
//! 负责处理用户相关的业务逻辑:
//! - 用户查询
//! - 用户创建/更新/删除
//! - 权限检查

use crate::error::AppError;
use crate::models::dto::{CreateUserRequest, UpdateUserRequest};
use crate::models::entities::User;
use crate::services::{AuthService, ServiceContext};
use std::sync::Arc;

/// 用户管理服务
pub struct UserService {
    ctx: Arc<ServiceContext>,
    auth_service: Arc<AuthService>,
}

impl UserService {
    /// 创建新的 UserService
    pub fn new(ctx: Arc<ServiceContext>, auth_service: Arc<AuthService>) -> Self {
        Self { ctx, auth_service }
    }

    /// 获取单个用户
    ///
    /// # 参数
    ///
    /// * `username` - 用户名
    pub async fn get_user(&self, username: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.ctx.pool)
            .await?
            .ok_or_else(|| AppError::not_found("User"))?;

        Ok(user)
    }

    /// 获取所有用户(仅管理员)
    ///
    /// # 参数
    ///
    /// * `is_admin` - 请求用户是否为管理员
    ///
    /// # 权限
    ///
    /// 需要管理员权限
    pub async fn get_all_users(&self, is_admin: bool) -> Result<Vec<User>, AppError> {
        // 权限检查
        if !is_admin {
            return Err(AppError::access_denied("Admin only"));
        }

        let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY username")
            .fetch_all(&self.ctx.pool)
            .await?;

        Ok(users)
    }

    /// 创建新用户(仅管理员)
    ///
    /// # 参数
    ///
    /// * `is_admin` - 请求用户是否为管理员
    /// * `request` - 创建用户请求
    ///
    /// # 权限
    ///
    /// 需要管理员权限
    pub async fn create_user(
        &self,
        is_admin: bool,
        request: CreateUserRequest,
    ) -> Result<(), AppError> {
        // 权限检查
        if !is_admin {
            return Err(AppError::access_denied("Admin only"));
        }

        // 检查用户名是否已存在
        let existing =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)")
                .bind(&request.username)
                .fetch_one(&self.ctx.pool)
                .await?;

        if existing {
            return Err(AppError::validation_error("Username already exists"));
        }

        // 使用 AuthService 注册用户
        self.auth_service.register(request).await?;

        Ok(())
    }

    /// 删除用户(仅管理员)
    ///
    /// # 参数
    ///
    /// * `is_admin` - 请求用户是否为管理员
    /// * `requester_username` - 请求用户的用户名(用于防止删除自己)
    /// * `username` - 要删除的用户名
    ///
    /// # 权限
    ///
    /// 需要管理员权限,且不能删除自己
    pub async fn delete_user(&self, is_admin: bool, requester_username: &str, username: &str) -> Result<(), AppError> {
        // 权限检查
        if !is_admin {
            return Err(AppError::access_denied("Admin only"));
        }

        // 不能删除自己
        if username == requester_username {
            return Err(AppError::validation_error("Cannot delete yourself"));
        }

        // 删除用户
        sqlx::query("DELETE FROM users WHERE username = ?")
            .bind(username)
            .execute(&self.ctx.pool)
            .await?;

        Ok(())
    }

    /// 更新用户信息(仅管理员)
    ///
    /// # 参数
    ///
    /// * `is_admin` - 请求用户是否为管理员
    /// * `username` - 要更新的用户名
    /// * `request` - 更新请求
    ///
    /// # 权限
    ///
    /// 需要管理员权限
    pub async fn update_user(
        &self,
        is_admin: bool,
        username: &str,
        request: UpdateUserRequest,
    ) -> Result<(), AppError> {
        // 权限检查
        if !is_admin {
            return Err(AppError::access_denied("Admin only"));
        }

        // 构建动态 UPDATE 查询
        let mut query_parts = Vec::new();

        if request.email.is_some() {
            query_parts.push("email = ?");
        }
        if request.max_bitrate.is_some() {
            query_parts.push("max_bitrate = ?");
        }
        if request.download_role.is_some() {
            query_parts.push("download_role = ?");
        }
        if request.upload_role.is_some() {
            query_parts.push("upload_role = ?");
        }
        if request.playlist_role.is_some() {
            query_parts.push("playlist_role = ?");
        }
        if request.cover_art_role.is_some() {
            query_parts.push("cover_art_role = ?");
        }
        if request.comment_role.is_some() {
            query_parts.push("comment_role = ?");
        }
        if request.podcast_role.is_some() {
            query_parts.push("podcast_role = ?");
        }
        if request.share_role.is_some() {
            query_parts.push("share_role = ?");
        }
        if request.video_conversion_role.is_some() {
            query_parts.push("video_conversion_role = ?");
        }
        if request.scrobbling_enabled.is_some() {
            query_parts.push("scrobbling_enabled = ?");
        }

        // 如果没有要更新的字段,直接返回
        if query_parts.is_empty() {
            return Ok(());
        }

        // 构建查询语句
        let query_str = format!(
            "UPDATE users SET {}, updated_at = CURRENT_TIMESTAMP WHERE username = ?",
            query_parts.join(", ")
        );

        // 绑定参数
        let mut query = sqlx::query(&query_str);

        if let Some(email) = request.email {
            query = query.bind(email);
        }
        if let Some(max_bitrate) = request.max_bitrate {
            query = query.bind(max_bitrate);
        }
        if let Some(download_role) = request.download_role {
            query = query.bind(download_role);
        }
        if let Some(upload_role) = request.upload_role {
            query = query.bind(upload_role);
        }
        if let Some(playlist_role) = request.playlist_role {
            query = query.bind(playlist_role);
        }
        if let Some(cover_art_role) = request.cover_art_role {
            query = query.bind(cover_art_role);
        }
        if let Some(comment_role) = request.comment_role {
            query = query.bind(comment_role);
        }
        if let Some(podcast_role) = request.podcast_role {
            query = query.bind(podcast_role);
        }
        if let Some(share_role) = request.share_role {
            query = query.bind(share_role);
        }
        if let Some(video_conversion_role) = request.video_conversion_role {
            query = query.bind(video_conversion_role);
        }
        if let Some(scrobbling_enabled) = request.scrobbling_enabled {
            query = query.bind(scrobbling_enabled);
        }

        query = query.bind(username);
        query.execute(&self.ctx.pool).await?;

        Ok(())
    }

    /// 修改密码
    ///
    /// # 参数
    ///
    /// * `is_admin` - 请求用户是否为管理员
    /// * `requester_username` - 请求用户的用户名
    /// * `target_username` - 目标用户名
    /// * `new_password` - 新密码
    ///
    /// # 权限
    ///
    /// 用户可以修改自己的密码,管理员可以修改任意用户的密码
    pub async fn change_password(
        &self,
        is_admin: bool,
        requester_username: &str,
        target_username: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        // 权限检查
        let can_change = is_admin || requester_username == target_username;

        if !can_change {
            return Err(AppError::access_denied(
                "Cannot change other user's password",
            ));
        }

        // 获取用户 ID
        let user_id = sqlx::query_scalar::<_, String>("SELECT id FROM users WHERE username = ?")
            .bind(target_username)
            .fetch_optional(&self.ctx.pool)
            .await?
            .ok_or_else(|| AppError::not_found("User"))?;

        // 使用 AuthService 修改密码
        self.auth_service
            .change_password(&user_id, new_password)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dto::CreateUserRequest;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // 创建用户表(使用完整的表结构)
        sqlx::query(
            "CREATE TABLE users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                email TEXT,
                is_admin BOOLEAN DEFAULT 0,
                max_bitrate INTEGER DEFAULT 320,
                download_role INTEGER DEFAULT 1,
                upload_role INTEGER DEFAULT 0,
                playlist_role INTEGER DEFAULT 1,
                cover_art_role INTEGER DEFAULT 1,
                comment_role INTEGER DEFAULT 0,
                podcast_role INTEGER DEFAULT 0,
                share_role INTEGER DEFAULT 1,
                video_conversion_role INTEGER DEFAULT 0,
                scrobbling_enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // 插入测试管理员
        sqlx::query(
            "INSERT INTO users (id, username, password, email, is_admin)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind("admin-id")
        .bind("admin")
        .bind("admin")
        .bind("admin@test.com")
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

        // 插入测试普通用户
        sqlx::query(
            "INSERT INTO users (id, username, password, email, is_admin)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind("user-id")
        .bind("user")
        .bind("password")
        .bind("user@test.com")
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    fn create_service(pool: SqlitePool) -> UserService {
        let ctx = Arc::new(ServiceContext::new(pool.clone()));
        let auth_service = Arc::new(AuthService::new(pool));
        UserService::new(ctx, auth_service)
    }

    #[tokio::test]
    async fn test_get_user() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let user = service.get_user("admin").await.unwrap();
        assert_eq!(user.username, "admin");
        assert!(user.is_admin);

        let result = service.get_user("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all_users_admin() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let users = service.get_all_users(true).await.unwrap();
        assert_eq!(users.len(), 2);
    }

    #[tokio::test]
    async fn test_get_all_users_non_admin() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let result = service.get_all_users(false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        // 管理员可以删除其他用户
        let result = service.delete_user(true, "admin", "user").await;
        assert!(result.is_ok());

        // 验证用户已删除
        let result = service.get_user("user").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_user_cannot_delete_self() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        // 不能删除自己
        let result = service.delete_user(true, "admin", "admin").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_user_non_admin() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        // 普通用户不能删除
        let result = service.delete_user(false, "user", "admin").await;
        assert!(result.is_err());
    }
}
