//! 数据库实体模块
//! 对应数据库表的完整结构,所有实体都实现 FromRow
#![allow(unused_imports)]

pub mod artist;
pub mod album;
pub mod song;
pub mod user;
pub mod playlist;
pub mod starred;
pub mod scrobble;
pub mod rating;

pub use artist::Artist;
pub use album::Album;
pub use song::Song;
pub use user::User;
pub use playlist::Playlist;
pub use starred::Starred;
pub use scrobble::Scrobble;
pub use rating::Rating;
