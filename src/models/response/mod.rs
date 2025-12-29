//! Subsonic API 响应模型模块
//! 所有响应结构都使用 Subsonic API 格式,不实现 FromRow

pub mod artist;
pub mod album;
pub mod song;
pub mod user;
pub mod playlist;
pub mod starred;
pub mod rating;
pub mod common;
pub mod search;
pub mod genre;
pub mod format;

// 导出常用类型
pub use artist::*;
pub use song::*;
pub use user::*;
pub use album::*;
pub use playlist::*;
pub use starred::*;
pub use rating::*;
pub use common::*;
pub use search::*;
pub use genre::*;
pub use format::ResponseFormat;

// 导出 XML 序列化 trait
pub use common::ToXml;
