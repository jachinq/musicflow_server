//! 数据库实体模块
//! 对应数据库表的完整结构,所有实体都实现 FromRow
#![allow(unused_imports)]

pub mod album;
pub mod artist;
pub mod play_queue;
pub mod playlist;
pub mod rating;
pub mod scrobble;
pub mod song;
pub mod starred;
pub mod user;

pub use album::Album;
pub use artist::Artist;
pub use play_queue::{PlayQueue, PlayQueueSong};
pub use playlist::Playlist;
pub use rating::Rating;
pub use scrobble::Scrobble;
pub use song::Song;
pub use starred::Starred;
pub use user::User;
