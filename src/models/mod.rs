//! 数据模型模块
//! 定义所有数据库实体和 API 响应模型

pub mod user;
pub mod artist;
pub mod album;
pub mod song;
pub mod playlist;
pub mod response;
pub mod starred;
pub mod scrobble;
pub mod rating;

// 导出数据库实体类型（使用 Db 前缀避免与API响应类型冲突）

// 导出 Subsonic API 响应类型
