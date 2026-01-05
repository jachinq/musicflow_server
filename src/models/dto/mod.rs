//! 数据传输对象模块
//! 用于数据库查询结果的中间层,所有 DTO 都实现 FromRow
#![allow(unused_imports)]

pub mod album;
pub mod artist;
pub mod playlist;
pub mod rating;
pub mod scrobble;
pub mod song;
pub mod starred;
pub mod user;

pub use album::*;
pub use artist::*;
pub use artist::ArtistStarredDto;
pub use playlist::*;
pub use rating::*;
pub use scrobble::*;
pub use song::*;
pub use starred::*;
pub use user::*;
