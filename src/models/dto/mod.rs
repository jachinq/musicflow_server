//! 数据传输对象模块
//! 用于数据库查询结果的中间层,所有 DTO 都实现 FromRow
#![allow(unused_imports)]

pub mod artist;
pub mod album;
pub mod song;
pub mod user;
pub mod playlist;
pub mod starred;
pub mod scrobble;
pub mod rating;

pub use artist::*;
pub use album::*;
pub use song::*;
pub use user::*;
pub use playlist::*;
pub use starred::*;
pub use scrobble::*;
pub use rating::*;
