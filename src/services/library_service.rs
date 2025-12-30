#![allow(dead_code)]
//! 库管理服务
//!
//! 负责处理音乐库相关的业务逻辑:
//! - Scrobble (播放记录)
//! - 收藏/取消收藏 (艺术家/专辑/歌曲)
//! - 评分

use crate::error::AppError;
use crate::models::dto::{AlbumDto, ArtistDto, SongDto};
use crate::services::ServiceContext;
use futures::FutureExt;
use std::sync::Arc;

/// 收藏项类型
#[derive(Debug, Clone, Copy)]
pub enum StarItemType {
    Artist,
    Album,
    Song,
}

/// 收藏数据集合
#[derive(Debug)]
pub struct StarredItems {
    pub artists: Vec<ArtistDto>,
    pub albums: Vec<AlbumDto>,
    pub songs: Vec<SongDto>,
}

/// 库管理服务
pub struct LibraryService {
    ctx: Arc<ServiceContext>,
}

impl LibraryService {
    /// 创建新的 LibraryService
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// 提交 scrobble (带事务保护)
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID
    /// * `song_id` - 歌曲 ID
    /// * `timestamp` - 播放时间戳
    /// * `submission` - 是否为正式提交 (true 会更新播放次数)
    ///
    /// # 事务保护
    ///
    /// 此方法使用事务确保:
    /// 1. scrobble 记录插入
    /// 2. 播放次数更新
    ///
    /// 两个操作要么全部成功,要么全部回滚
    pub async fn submit_scrobble(
        &self,
        user_id: &str,
        song_id: &str,
        timestamp: i64,
        submission: bool,
    ) -> Result<(), AppError> {
        let user_id = user_id.to_string();
        let song_id = song_id.to_string();

        self.ctx
            .transaction(|tx| {
                async move {
                    // 插入 scrobble 记录
                    sqlx::query(
                        "INSERT INTO scrobbles (id, user_id, song_id, timestamp, submission, created_at)
                         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
                    )
                    .bind(uuid::Uuid::new_v4().to_string())
                    .bind(&user_id)
                    .bind(&song_id)
                    .bind(timestamp)
                    .bind(submission)
                    .execute(&mut **tx)
                    .await?;

                    // 如果是正式提交,更新播放计数
                    if submission {
                        sqlx::query("UPDATE songs SET play_count = play_count + 1 WHERE id = ?")
                            .bind(&song_id)
                            .execute(&mut **tx)
                            .await?;

                       // 查询专辑播放次数
                        let album_id: Option<String> = sqlx::query_scalar("SELECT album_id FROM songs WHERE id = ?")
                            .bind(&song_id)
                            .fetch_optional(&mut **tx)
                            .await?;

                        tracing::info!("Album play count update: {:?}", album_id);
                        if let Some(album_id) = album_id {
                            sqlx::query("UPDATE albums SET play_count = play_count + 1 WHERE id = ?",)
                                .bind(&album_id)
                                .execute(&mut **tx)
                                .await?;
                        }

                    }

                    Ok(())
                }
                .boxed()
            })
            .await
    }

    /// 收藏项目
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID
    /// * `item_type` - 收藏类型 (艺术家/专辑/歌曲)
    /// * `item_id` - 项目 ID
    pub async fn star_item(
        &self,
        user_id: &str,
        item_type: StarItemType,
        item_id: &str,
    ) -> Result<(), AppError> {
        let id = uuid::Uuid::new_v4().to_string();

        let query = match item_type {
            StarItemType::Artist => {
                "INSERT OR IGNORE INTO starred (id, user_id, artist_id, created_at)
                 VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
            }
            StarItemType::Album => {
                "INSERT OR IGNORE INTO starred (id, user_id, album_id, created_at)
                 VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
            }
            StarItemType::Song => {
                "INSERT OR IGNORE INTO starred (id, user_id, song_id, created_at)
                 VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
            }
        };

        sqlx::query(query)
            .bind(&id)
            .bind(user_id)
            .bind(item_id)
            .execute(&self.ctx.pool)
            .await?;

        Ok(())
    }

    /// 取消收藏项目
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID
    /// * `item_type` - 收藏类型 (艺术家/专辑/歌曲)
    /// * `item_id` - 项目 ID
    pub async fn unstar_item(
        &self,
        user_id: &str,
        item_type: StarItemType,
        item_id: &str,
    ) -> Result<(), AppError> {
        let query = match item_type {
            StarItemType::Artist => "DELETE FROM starred WHERE user_id = ? AND artist_id = ?",
            StarItemType::Album => "DELETE FROM starred WHERE user_id = ? AND album_id = ?",
            StarItemType::Song => "DELETE FROM starred WHERE user_id = ? AND song_id = ?",
        };

        sqlx::query(query)
            .bind(user_id)
            .bind(item_id)
            .execute(&self.ctx.pool)
            .await?;

        Ok(())
    }

    /// 设置评分
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID
    /// * `item_id` - 项目 ID (通常是歌曲 ID)
    /// * `rating` - 评分 (1-5)
    ///
    /// # 错误
    ///
    /// 如果评分不在 1-5 范围内,返回 ValidationError
    pub async fn set_rating(
        &self,
        user_id: &str,
        item_id: &str,
        rating: i32,
    ) -> Result<(), AppError> {
        if !(1..=5).contains(&rating) {
            return Err(AppError::validation_error("Rating must be between 1 and 5"));
        }

        sqlx::query(
            "INSERT OR REPLACE INTO ratings (id, user_id, song_id, rating, created_at, updated_at)
             VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(user_id)
        .bind(item_id)
        .bind(rating)
        .execute(&self.ctx.pool)
        .await?;

        Ok(())
    }

    /// 获取评分
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID
    /// * `item_id` - 项目 ID
    ///
    /// # 返回
    ///
    /// 返回评分值,如果未评分则返回 None
    pub async fn get_rating(&self, user_id: &str, item_id: &str) -> Result<Option<i32>, AppError> {
        let rating = sqlx::query_scalar::<_, i32>(
            "SELECT rating FROM ratings WHERE user_id = ? AND song_id = ?",
        )
        .bind(user_id)
        .bind(item_id)
        .fetch_optional(&self.ctx.pool)
        .await?;

        Ok(rating)
    }

    /// 获取用户收藏的所有项目 (并行查询)
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID
    ///
    /// # 性能优化
    ///
    /// 使用 tokio::try_join! 并行查询三个表,提升性能
    pub async fn get_starred_items(&self, user_id: &str) -> Result<StarredItems, AppError> {
        let user_id = user_id.to_string();

        // 并行查询三个表
        let (artists, albums, songs) = tokio::try_join!(
            self.get_starred_artists(&user_id),
            self.get_starred_albums(&user_id),
            self.get_starred_songs(&user_id),
        )?;

        Ok(StarredItems {
            artists,
            albums,
            songs,
        })
    }

    /// 获取收藏的艺术家 (私有方法)
    async fn get_starred_artists(&self, user_id: &str) -> Result<Vec<ArtistDto>, AppError> {
        let artists = sqlx::query_as::<_, ArtistDto>(
            "SELECT a.id, a.name
             FROM starred s
             JOIN artists a ON s.artist_id = a.id
             WHERE s.user_id = ? AND s.artist_id IS NOT NULL",
        )
        .bind(user_id)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok(artists)
    }

    /// 获取收藏的专辑 (私有方法)
    async fn get_starred_albums(&self, user_id: &str) -> Result<Vec<AlbumDto>, AppError> {
        let albums = sqlx::query_as::<_, AlbumDto>(
            "SELECT a.id, a.name, ar.name as artist, a.year, a.song_count
             FROM starred s
             JOIN albums a ON s.album_id = a.id
             JOIN artists ar ON a.artist_id = ar.id
             WHERE s.user_id = ? AND s.album_id IS NOT NULL",
        )
        .bind(user_id)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok(albums)
    }

    /// 获取收藏的歌曲 (私有方法)
    async fn get_starred_songs(&self, user_id: &str) -> Result<Vec<SongDto>, AppError> {
        let songs = sqlx::query_as::<_, SongDto>(
            "SELECT s.id, s.title, ar.name as artist, al.name as album, s.duration, s.content_type
             FROM starred st
             JOIN songs s ON st.song_id = s.id
             JOIN albums al ON s.album_id = al.id
             JOIN artists ar ON s.artist_id = ar.id
             WHERE st.user_id = ? AND st.song_id IS NOT NULL",
        )
        .bind(user_id)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok(songs)
    }

    /// 获取音乐库中的歌曲总数
    ///
    /// # 返回值
    ///
    /// 返回数据库中歌曲表的总记录数
    pub async fn get_song_count(&self) -> Result<usize, AppError> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM songs")
            .fetch_one(&self.ctx.pool)
            .await?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // 创建测试表
        sqlx::query(
            "CREATE TABLE users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE songs (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                play_count INTEGER DEFAULT 0
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE scrobbles (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                song_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                submission BOOLEAN NOT NULL,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE starred (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                artist_id TEXT,
                album_id TEXT,
                song_id TEXT,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE ratings (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                song_id TEXT NOT NULL,
                rating INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // 插入测试数据
        sqlx::query("INSERT INTO users (id, username) VALUES (?, ?)")
            .bind("user1")
            .bind("testuser")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO songs (id, title, play_count) VALUES (?, ?, ?)")
            .bind("song1")
            .bind("Test Song")
            .bind(0)
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    fn create_service(pool: SqlitePool) -> LibraryService {
        let ctx = Arc::new(ServiceContext::new(pool));
        LibraryService::new(ctx)
    }

    #[tokio::test]
    async fn test_submit_scrobble_with_submission() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        // 提交 scrobble
        let result = service
            .submit_scrobble("user1", "song1", 1234567890, true)
            .await;
        assert!(result.is_ok());

        // 验证 scrobble 记录已插入
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scrobbles")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);

        // 验证播放次数已更新
        let play_count: i32 = sqlx::query_scalar("SELECT play_count FROM songs WHERE id = ?")
            .bind("song1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(play_count, 1);
    }

    #[tokio::test]
    async fn test_submit_scrobble_without_submission() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        // 提交 scrobble (不更新播放次数)
        let result = service
            .submit_scrobble("user1", "song1", 1234567890, false)
            .await;
        assert!(result.is_ok());

        // 验证播放次数未更新
        let play_count: i32 = sqlx::query_scalar("SELECT play_count FROM songs WHERE id = ?")
            .bind("song1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(play_count, 0);
    }

    #[tokio::test]
    async fn test_star_and_unstar_song() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        // 收藏歌曲
        let result = service
            .star_item("user1", StarItemType::Song, "song1")
            .await;
        assert!(result.is_ok());

        // 验证已收藏
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM starred WHERE user_id = ? AND song_id = ?")
                .bind("user1")
                .bind("song1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1);

        // 取消收藏
        let result = service
            .unstar_item("user1", StarItemType::Song, "song1")
            .await;
        assert!(result.is_ok());

        // 验证已取消
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM starred WHERE user_id = ? AND song_id = ?")
                .bind("user1")
                .bind("song1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_set_and_get_rating() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        // 设置评分
        let result = service.set_rating("user1", "song1", 5).await;
        assert!(result.is_ok());

        // 获取评分
        let rating = service.get_rating("user1", "song1").await.unwrap();
        assert_eq!(rating, Some(5));

        // 更新评分
        let result = service.set_rating("user1", "song1", 3).await;
        assert!(result.is_ok());

        // 验证已更新
        let rating = service.get_rating("user1", "song1").await.unwrap();
        assert_eq!(rating, Some(3));
    }

    #[tokio::test]
    async fn test_set_rating_invalid() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        // 评分太低
        let result = service.set_rating("user1", "song1", 0).await;
        assert!(result.is_err());

        // 评分太高
        let result = service.set_rating("user1", "song1", 6).await;
        assert!(result.is_err());
    }
}
