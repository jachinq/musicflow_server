//! Music Flow Server Library
//!
//! 基于 Subsonic API 1.16.1 的音乐流媒体服务器

// 模块声明
pub mod config;
pub mod database;
pub mod error;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod response;
pub mod services;
pub mod utils;

// 重新导出常用类型
pub use config::AppConfig;
pub use database::{get_db_pool, run_migrations};
pub use error::AppError;
pub use models::*;

/// 应用版本
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// API 版本 (Subsonic 兼容)
pub const API_VERSION: &str = "1.16.1";
