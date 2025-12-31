//! Subsonic API 响应模型模块
//! 所有响应结构都使用 Subsonic API 格式,不实现 FromRow

pub mod album;
pub mod artist;
pub mod common;
pub mod format;
pub mod genre;
pub mod playlist;
pub mod rating;
pub mod search;
pub mod song;
pub mod starred;
pub mod user;

// 导出常用类型
pub use album::*;
pub use artist::*;
pub use common::*;
pub use format::ResponseFormat;
pub use genre::*;
pub use playlist::*;
pub use rating::*;
pub use search::*;
pub use song::*;
pub use starred::*;
pub use user::*;

// 导出 XML 序列化 trait
pub use common::ToXml;
