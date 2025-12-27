//! 数据模型模块
//!
//! 模型层级:
//! - entities: 数据库实体 (完整表结构,实现 FromRow)
//! - dto: 数据传输对象 (查询结果,实现 FromRow)
//! - response: API 响应模型 (Subsonic 格式,不实现 FromRow)
//!
//! 数据流: Database → DTO (FromRow) → Response (Into/From) → JSON

pub mod entities;
pub mod dto;
pub mod response;