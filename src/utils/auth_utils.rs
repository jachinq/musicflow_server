//! 认证工具函数
#![allow(dead_code)]

use crate::error::AppError;

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

    use super::*;

    #[test]
    fn test_require_admin() {
        assert!(require_admin(true).is_ok());
        assert!(require_admin(false).is_err());
    }
}