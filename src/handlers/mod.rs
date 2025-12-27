pub mod auth;
pub mod browsing;
pub mod library;
pub mod search;
pub mod system;
pub mod stream;
pub mod playlist;
pub mod user;

// 注意：为了避免命名冲突，不使用glob导入，而是直接使用模块名