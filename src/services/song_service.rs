//! 歌曲信息服务
//!
//! 负责处理歌曲相关的业务逻辑:
//! - 将 SongDetailDto 丰富为 ComplexSongDto
//! - 批量处理用户相关信息（rating、starred、suffix）
//! - 提供统一的歌曲查询接口

use crate::error::AppError;
use crate::models::dto::{ComplexSongDto, SongDetailDto};
use crate::services::ServiceContext;
use crate::utils::{image_utils, sql_utils};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

/// 歌曲信息服务
pub struct SongService {
    ctx: Arc<ServiceContext>,
}

impl SongService {
    /// 创建新的 SongService
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// 将 SongDetailDto 列表丰富为 ComplexSongDto
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID（用于查询用户相关信息）
    /// * `songs` - 歌曲基础信息列表
    ///
    /// # 性能优化
    ///
    /// - 批量查询 ratings 和 starred（避免 N+1）
    /// - 使用 HashMap 建立索引（O(1) 查找）
    pub async fn enrich_songs(
        &self,
        user_id: &str,
        songs: Vec<SongDetailDto>,
    ) -> Result<Vec<ComplexSongDto>, AppError> {
        if songs.is_empty() {
            return Ok(vec![]);
        }

        // 提取所有 song_id
        let song_ids: Vec<String> = songs.iter().map(|s| s.id.clone()).collect();

        // 批量查询 ratings（使用 IN 子句）
        let rating_list = self.get_ratings_batch(user_id, &song_ids).await?;
        let rating_map: HashMap<String, i32> = rating_list
            .into_iter()
            .map(|(rating, song_id)| (song_id, rating))
            .collect();

        // 批量查询 starred（使用 IN 子句）
        let starred_list = self.get_starred_batch(user_id, &song_ids).await?;
        let starred_set: HashSet<String> = starred_list.into_iter().collect();

        // 组装 ComplexSongDto
        let complex_songs = songs
            .into_iter()
            .map(|song| {
                let suffix = song.path.as_ref().map(|p| {
                    image_utils::get_content_type(Path::new(p))
                });

                let user_rating = rating_map.get(&song.id).copied();
                let starred = if starred_set.contains(&song.id) {
                    Some(true)
                } else {
                    None
                };

                ComplexSongDto {
                    song,
                    user_rating,
                    starred,
                    suffix,
                }
            })
            .collect();

        Ok(complex_songs)
    }

    /// 通过 song_id 获取单个 ComplexSongDto
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID
    /// * `song_id` - 歌曲 ID
    pub async fn get_complex_song(
        &self,
        user_id: &str,
        song_id: &str,
    ) -> Result<ComplexSongDto, AppError> {
        // 查询歌曲基础信息
        let song = sqlx::query_as::<_, SongDetailDto>(&format!(
            "{} WHERE s.id = ?",
            sql_utils::detail_sql()
        ))
        .bind(song_id)
        .fetch_optional(&self.ctx.pool)
        .await?
        .ok_or_else(|| AppError::not_found("Song"))?;

        // 使用 enrich_songs 复用逻辑
        let mut complex_songs = self.enrich_songs(user_id, vec![song]).await?;

        Ok(complex_songs.remove(0))
    }


    /// 通过 song_ids 批量获取 ComplexSongDto
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID
    /// * `song_ids` - 歌曲 ID 列表
    pub async fn get_complex_songs_by_ids(
        &self,
        user_id: &str,
        song_ids: &[String],
    ) -> Result<Vec<ComplexSongDto>, AppError> {
        if song_ids.is_empty() {
            return Ok(vec![]);
        }

        // 构建 IN 子句
        let placeholders = song_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!("{} WHERE s.id IN ({})", sql_utils::detail_sql(), placeholders);

        // 查询歌曲
        
        let mut query_builder = sqlx::query_as::<_, SongDetailDto>(&query);

        for song_id in song_ids {
            query_builder = query_builder.bind(song_id);
        }
        let songs = query_builder
        .fetch_all(&self.ctx.pool)
        .await?;

        // 使用 enrich_songs 复用逻辑
        self.enrich_songs(user_id, songs).await
    }


    /// 批量查询 ratings（私有方法）
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID
    /// * `song_ids` - 歌曲 ID 列表
    async fn get_ratings_batch(
        &self,
        user_id: &str,
        song_ids: &[String],
    ) -> Result<Vec<(i32, String)>, AppError> {
        if song_ids.is_empty() {
            return Ok(vec![]);
        }

        // 构建 IN 子句
        let placeholders = song_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT rating, song_id FROM ratings WHERE user_id = ? AND song_id IN ({})",
            placeholders
        );

        let mut query_builder = sqlx::query_as::<_, (i32, String)>(&query);
        query_builder = query_builder.bind(user_id);
        for song_id in song_ids {
            query_builder = query_builder.bind(song_id);
        }

        let results = query_builder.fetch_all(&self.ctx.pool).await?;
        Ok(results)
    }

    /// 批量查询 starred（私有方法）
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID
    /// * `song_ids` - 歌曲 ID 列表
    async fn get_starred_batch(
        &self,
        user_id: &str,
        song_ids: &[String],
    ) -> Result<Vec<String>, AppError> {
        if song_ids.is_empty() {
            return Ok(vec![]);
        }

        // 构建 IN 子句
        let placeholders = song_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT song_id FROM starred WHERE user_id = ? AND song_id IN ({})",
            placeholders
        );

        let mut query_builder = sqlx::query_scalar::<_, String>(&query);
        query_builder = query_builder.bind(user_id);
        for song_id in song_ids {
            query_builder = query_builder.bind(song_id);
        }

        let results = query_builder.fetch_all(&self.ctx.pool).await?;
        Ok(results)
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
            "CREATE TABLE artists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE albums (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                artist_id TEXT NOT NULL,
                cover_art_path TEXT
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE songs (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist_id TEXT NOT NULL,
                album_id TEXT NOT NULL,
                duration INTEGER DEFAULT 0,
                bit_rate INTEGER,
                track_number INTEGER,
                disc_number INTEGER,
                year INTEGER,
                genre TEXT,
                file_path TEXT,
                file_size INTEGER,
                content_type TEXT,
                play_count INTEGER DEFAULT 0
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
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
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
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // 插入测试数据
        sqlx::query("INSERT INTO artists (id, name) VALUES ('artist1', 'Test Artist')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO albums (id, name, artist_id, cover_art_path) VALUES ('album1', 'Test Album', 'artist1', '/path/to/cover.jpg')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO songs (id, title, artist_id, album_id, duration, track_number, genre, play_count, file_path)
             VALUES ('song1', 'Song 1', 'artist1', 'album1', 180, 1, 'Rock', 50, '/path/to/song1.mp3')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO songs (id, title, artist_id, album_id, duration, track_number, genre, play_count, file_path)
             VALUES ('song2', 'Song 2', 'artist1', 'album1', 120, 2, 'Rock', 30, '/path/to/song2.mp3')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO ratings (id, user_id, song_id, rating)
             VALUES ('rating1', 'user1', 'song1', 5)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO starred (id, user_id, song_id)
             VALUES ('star1', 'user1', 'song2')",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    fn create_service(pool: SqlitePool) -> SongService {
        let ctx = Arc::new(ServiceContext::new(pool));
        SongService::new(ctx)
    }

    #[tokio::test]
    async fn test_enrich_songs_empty() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let result = service.enrich_songs("user1", vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_enrich_songs_with_rating_and_starred() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        // 查询歌曲基础信息
        let songs = sqlx::query_as::<_, SongDetailDto>(
            "SELECT s.id, s.title, ar.name as artist, s.artist_id, al.name as album, s.album_id,
                    s.track_number, s.disc_number, s.duration, s.bit_rate, s.genre, s.year,
                    s.content_type, s.file_path as path, al.cover_art_path as cover_art,
                    s.file_size, s.play_count
             FROM songs s
             JOIN albums al ON s.album_id = al.id
             JOIN artists ar ON s.artist_id = ar.id
             ORDER BY s.track_number",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(songs.len(), 2);

        // 丰富歌曲信息
        let complex_songs = service.enrich_songs("user1", songs).await.unwrap();

        assert_eq!(complex_songs.len(), 2);

        // 验证 song1 有 rating
        assert_eq!(complex_songs[0].song.title, "Song 1");
        assert_eq!(complex_songs[0].user_rating, Some(5));
        assert_eq!(complex_songs[0].starred, None);
        assert!(complex_songs[0].suffix.is_some());

        // 验证 song2 有 starred
        assert_eq!(complex_songs[1].song.title, "Song 2");
        assert_eq!(complex_songs[1].user_rating, None);
        assert_eq!(complex_songs[1].starred, Some(true));
        assert!(complex_songs[1].suffix.is_some());
    }

    #[tokio::test]
    async fn test_get_complex_song() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let complex_song = service.get_complex_song("user1", "song1").await.unwrap();

        assert_eq!(complex_song.song.title, "Song 1");
        assert_eq!(complex_song.user_rating, Some(5));
        assert_eq!(complex_song.starred, None);
        assert!(complex_song.suffix.is_some());
    }

    #[tokio::test]
    async fn test_get_complex_song_not_found() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let result = service.get_complex_song("user1", "nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_ratings_batch() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let song_ids = vec!["song1".to_string(), "song2".to_string()];
        let ratings = service.get_ratings_batch("user1", &song_ids).await.unwrap();

        assert_eq!(ratings.len(), 1);
        assert_eq!(ratings[0].0, 5); // rating
        assert_eq!(ratings[0].1, "song1"); // song_id
    }

    #[tokio::test]
    async fn test_get_starred_batch() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let song_ids = vec!["song1".to_string(), "song2".to_string()];
        let starred = service.get_starred_batch("user1", &song_ids).await.unwrap();

        assert_eq!(starred.len(), 1);
        assert_eq!(starred[0], "song2");
    }
}
