//! 应用配置
#![allow(dead_code)]

use dotenvy::dotenv;
use std::env;
use std::path::PathBuf;

/// 应用配置结构体
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
    pub host: String,
    pub music_library_path: PathBuf,
    pub rust_log: String,
    pub app_name: String,
    pub app_version: String,
}

impl AppConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, anyhow::Error> {
        // 加载 .env 文件
        dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:data/music_flow.db".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "4040".to_string())
                .parse()
                .unwrap_or(4040),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            music_library_path: PathBuf::from(
                env::var("MUSIC_LIBRARY_PATH").unwrap_or_else(|_| "/path/to/music".to_string()),
            ),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            app_name: env::var("APP_NAME").unwrap_or_else(|_| "MusicFlowServer".to_string()),
            app_version: env::var("APP_VERSION").unwrap_or_else(|_| "1.0.0".to_string()),
        })
    }

    /// 获取服务器地址
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// 检查音乐库路径是否存在
    pub fn validate_music_library(&self) -> Result<(), anyhow::Error> {
        if !self.music_library_path.exists() {
            return Err(anyhow::anyhow!(
                "Music library path does not exist: {}",
                self.music_library_path.display()
            ));
        }
        if !self.music_library_path.is_dir() {
            return Err(anyhow::anyhow!(
                "Music library path is not a directory: {}",
                self.music_library_path.display()
            ));
        }
        Ok(())
    }

    /// 创建测试配置
    #[cfg(test)]
    pub fn test_config() -> Self {
        Self {
            database_url: "sqlite::memory:".to_string(),
            port: 4040,
            host: "127.0.0.1".to_string(),
            music_library_path: PathBuf::from("/tmp/test_music"),
            rust_log: "error".to_string(),
            app_name: "TestServer".to_string(),
            app_version: "0.1.0".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        // 设置测试环境变量
        std::env::set_var("DATABASE_URL", "sqlite:test.db");
        std::env::set_var("PORT", "8080");
        std::env::set_var("HOST", "0.0.0.0");
        std::env::set_var("MUSIC_LIBRARY_PATH", "/tmp/music");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.database_url, "sqlite:test.db");
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");

        // 清理
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("PORT");
        std::env::remove_var("HOST");
        std::env::remove_var("MUSIC_LIBRARY_PATH");
    }

    #[test]
    fn test_server_address() {
        let config = AppConfig {
            database_url: "sqlite:test.db".to_string(),
            port: 4040,
            host: "127.0.0.1".to_string(),
            music_library_path: PathBuf::from("/tmp/music"),
            rust_log: "info".to_string(),
            app_name: "Test".to_string(),
            app_version: "1.0.0".to_string(),
        };

        assert_eq!(config.server_address(), "127.0.0.1:4040");
    }
}
