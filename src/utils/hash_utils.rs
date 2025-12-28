//! 哈希工具函数
#![allow(dead_code)]

use md5;

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