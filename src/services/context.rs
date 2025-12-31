//! Service 层上下文和事务支持

use crate::error::AppError;
use futures::future::BoxFuture;
use sqlx::{Sqlite, SqlitePool, Transaction};

/// Service 层共享上下文
///
/// 提供连接池管理和统一的事务支持
pub struct ServiceContext {
    pub pool: SqlitePool,
}

impl ServiceContext {
    /// 创建新的 ServiceContext
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 执行事务
    ///
    /// # 示例
    ///
    /// ```rust
    /// service.ctx.transaction(|tx| async move {
    ///     sqlx::query("INSERT INTO ...").execute(&mut **tx).await?;
    ///     sqlx::query("UPDATE ...").execute(&mut **tx).await?;
    ///     Ok(())
    /// }.boxed()).await?;
    /// ```
    ///
    /// # 特性
    ///
    /// - 自动管理事务生命周期
    /// - 失败时自动回滚
    /// - 成功时自动提交
    pub async fn transaction<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: for<'a> FnOnce(&'a mut Transaction<'_, Sqlite>) -> BoxFuture<'a, Result<T, AppError>>,
    {
        let mut tx = self.pool.begin().await?;
        let result = f(&mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // 创建测试表
        sqlx::query(
            "CREATE TABLE test_table (
                id TEXT PRIMARY KEY,
                value INTEGER
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_transaction_success() {
        let pool = setup_test_db().await;
        let ctx = ServiceContext::new(pool.clone());

        // 执行事务
        let result = ctx
            .transaction(|tx| {
                async move {
                    sqlx::query("INSERT INTO test_table (id, value) VALUES (?, ?)")
                        .bind("test1")
                        .bind(100)
                        .execute(&mut **tx)
                        .await?;

                    sqlx::query("INSERT INTO test_table (id, value) VALUES (?, ?)")
                        .bind("test2")
                        .bind(200)
                        .execute(&mut **tx)
                        .await?;

                    Ok::<(), AppError>(())
                }
                .boxed()
            })
            .await;

        assert!(result.is_ok());

        // 验证数据已提交
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_table")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let pool = setup_test_db().await;
        let ctx = ServiceContext::new(pool.clone());

        // 执行会失败的事务
        let result = ctx
            .transaction(|tx| {
                async move {
                    sqlx::query("INSERT INTO test_table (id, value) VALUES (?, ?)")
                        .bind("test1")
                        .bind(100)
                        .execute(&mut **tx)
                        .await?;

                    // 故意插入重复的主键导致失败
                    sqlx::query("INSERT INTO test_table (id, value) VALUES (?, ?)")
                        .bind("test1") // 重复的 ID
                        .bind(200)
                        .execute(&mut **tx)
                        .await?;

                    Ok::<(), AppError>(())
                }
                .boxed()
            })
            .await;

        assert!(result.is_err());

        // 验证数据已回滚
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_table")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
