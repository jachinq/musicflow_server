//! 数据库模块
//! 提供数据库连接池和初始化功能

pub mod connection;

pub use connection::{get_db_pool, run_migrations, DbPool};