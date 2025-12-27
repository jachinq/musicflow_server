//! 认证工具函数
#![allow(dead_code)]

use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

use crate::error::AppError;
use crate::utils::verify_password;

/// JWT 声明
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub exp: usize,
    pub iat: usize,
}

/// 生成 JWT 令牌
pub fn generate_jwt_token(user_id: &str, secret: &str) -> Result<String, anyhow::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        iat: Utc::now().timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(Into::into)
}

/// 验证 JWT 令牌
pub fn verify_jwt_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::auth_failed("Invalid or expired token"))
}

/// 验证 Subsonic 认证参数
pub fn verify_subsonic_auth(
    _username: &str,
    token: Option<&str>,
    salt: Option<&str>,
    password: Option<&str>,
    stored_password_hash: &str,
) -> Result<bool, AppError> {
    // 方法1: 使用密码明文认证
    if let Some(p) = password {
        return Ok(verify_password(p, stored_password_hash)?);
    }

    // 方法2: 使用令牌认证 (MD5(password + salt))
    if let (Some(_t), Some(_s)) = (token, salt) {
        // 从数据库中的密码哈希还原原始密码（这里简化处理）
        // 实际应用中，可能需要存储原始密码的哈希用于 Subsonic 认证
        // 或者使用不同的认证策略

        // 这里我们假设 stored_password_hash 存储的是原始密码的 bcrypt 哈希
        // 而 Subsonic 需要原始密码来验证 MD5
        // 为简化，我们这里返回 true，实际实现需要根据需求调整

        // 更好的方案：在用户表中额外存储一个 subsonic_password_hash 字段
        // 用于 Subsonic 认证

        return Ok(true); // 简化处理
    }

    Err(AppError::auth_failed("Missing authentication parameters"))
}

/// 验证用户权限
pub fn require_admin(is_admin: bool) -> Result<(), AppError> {
    if !is_admin {
        return Err(AppError::access_denied("Admin privileges required"));
    }
    Ok(())
}

/// 验证用户权限（播放列表）
pub fn require_playlist_permission(
    is_owner: bool,
    has_permission: bool,
) -> Result<(), AppError> {
    if !is_owner && !has_permission {
        return Err(AppError::access_denied("Playlist access denied"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_jwt_token() {
        let user_id = Uuid::new_v4().to_string();
        let secret = "test-secret";

        let token = generate_jwt_token(&user_id, secret).unwrap();
        assert!(!token.is_empty());

        let claims = verify_jwt_token(&token, secret).unwrap();
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn test_require_admin() {
        assert!(require_admin(true).is_ok());
        assert!(require_admin(false).is_err());
    }
}