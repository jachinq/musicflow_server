//! 哈希工具函数
#![allow(dead_code)]

use bcrypt::{hash, verify, DEFAULT_COST};
use md5;

/// 使用 bcrypt 哈希密码
pub fn hash_password(password: &str) -> Result<String, anyhow::Error> {
    hash(password, DEFAULT_COST).map_err(Into::into)
}

/// 验证 bcrypt 哈希密码
pub fn verify_password(password: &str, hash: &str) -> Result<bool, anyhow::Error> {
    verify(password, hash).map_err(Into::into)
}

/// 生成 Subsonic MD5 令牌 (MD5(password + salt))
pub fn generate_subsonic_token(password: &str, salt: &str) -> String {
    let combined = format!("{}{}", password, salt);
    format!("{:x}", md5::compute(combined))
}

/// 生成随机 Salt
pub fn generate_salt() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen::<[u8; 16]>()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn test_subsonic_token() {
        let password = "password";
        let salt = "salt123";
        let token = generate_subsonic_token(password, salt);
        // MD5("passwordsalt123")
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        assert_eq!(salt1.len(), 32);
        assert_ne!(salt1, salt2); // 应该是随机的
    }
}