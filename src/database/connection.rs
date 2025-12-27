//! 数据库连接管理

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;

/// 数据库连接池类型
pub type DbPool = SqlitePool;

/// 获取数据库连接池
///
/// # Arguments
///
/// * `database_url` - 数据库 URL (例如: "sqlite:music_flow.db")
///
/// # Returns
///
/// * `Result<DbPool, sqlx::Error>` - 数据库连接池或错误
///
/// # Example
///
/// ```rust
/// //let pool = get_db_pool("sqlite:music_flow.db").await?;
/// ```
pub async fn get_db_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(10) // SQLite 通常只需要一个连接，但为了并发可以适当增加
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await
}

/// 运行数据库迁移
///
/// # Arguments
///
/// * `pool` - 数据库连接池
///
/// # Returns
///
/// * `Result<(), sqlx::Error>` - 成功或错误
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        // 使用内存数据库进行测试
        let pool = get_db_pool("sqlite::memory:").await.unwrap();

        // 测试简单查询
        let result: i32 = sqlx::query_scalar("SELECT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result, 1);
    }
}