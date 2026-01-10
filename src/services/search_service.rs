//! 搜索功能服务
//!
//! 负责处理搜索相关的业务逻辑:
//! - 艺术家/专辑/歌曲搜索
//! - 并行查询优化
//! - 统一搜索接口
#![allow(dead_code)]

use crate::error::AppError;
use crate::models::dto::{AlbumDetailDto, AlbumDto, ArtistDto, ComplexSongDto, SongDetailDto};
use crate::services::{ServiceContext, SongService};
use crate::utils::sql_utils;
use std::sync::Arc;

/// 搜索结果
#[derive(Debug)]
pub struct SearchResults {
    pub artists: Vec<ArtistDto>,
    pub albums: Vec<AlbumDetailDto>,
    pub songs: Vec<ComplexSongDto>,
}

/// 搜索结果 (简化版,用于 search2)
#[derive(Debug)]
pub struct SearchResults2 {
    pub artists: Vec<ArtistDto>,
    pub albums: Vec<AlbumDto>,
    pub songs: Vec<ComplexSongDto>,
}

/// 搜索参数
#[derive(Debug, Clone)]
pub struct SearchParams {
    pub query: String,
    pub artist_count: i32,
    pub artist_offset: i32,
    pub album_count: i32,
    pub album_offset: i32,
    pub song_count: i32,
    pub song_offset: i32,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            artist_count: 20,
            artist_offset: 0,
            album_count: 20,
            album_offset: 0,
            song_count: 20,
            song_offset: 0,
        }
    }
}

/// 搜索功能服务
pub struct SearchService {
    ctx: Arc<ServiceContext>,
}

impl SearchService {
    /// 创建新的 SearchService
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// 搜索所有类型 (并行查询)
    ///
    /// # 参数
    ///
    /// * `params` - 搜索参数
    ///
    /// # 性能优化
    ///
    /// 使用 tokio::try_join! 并行执行三个独立查询,提升性能
    pub async fn search_all(&self, user_id: &str, params: SearchParams) -> Result<SearchResults, AppError> {
        let query = params.query.clone();

        // 并行搜索三个表
        let (artists, albums, songs) = tokio::try_join!(
            self.search_artists(&query, params.artist_count, params.artist_offset),
            self.search_albums_detailed(&query, params.album_count, params.album_offset),
            self.search_songs(user_id, &query, params.song_count, params.song_offset),
        )?;

        Ok(SearchResults {
            artists,
            albums,
            songs,
        })
    }

    /// 搜索所有类型 (简化版,并行查询)
    ///
    /// # 参数
    ///
    /// * `params` - 搜索参数
    ///
    /// # 性能优化
    ///
    /// 与 search_all 相同使用并行查询,但返回简化的专辑信息
    pub async fn search_all_simple(
        &self,
        user_id: &str,
        params: SearchParams,
    ) -> Result<SearchResults2, AppError> {
        let query = params.query.clone();

        // 并行搜索三个表
        let (artists, albums, songs) = tokio::try_join!(
            self.search_artists(&query, params.artist_count, params.artist_offset),
            self.search_albums_simple(&query, params.album_count, params.album_offset),
            self.search_songs(user_id, &query, params.song_count, params.song_offset),
        )?;

        Ok(SearchResults2 {
            artists,
            albums,
            songs,
        })
    }

    /// 搜索艺术家
    ///
    /// # 参数
    ///
    /// * `query` - 搜索关键词
    /// * `count` - 返回数量
    /// * `offset` - 偏移量
    async fn search_artists(
        &self,
        query: &str,
        count: i32,
        offset: i32,
    ) -> Result<Vec<ArtistDto>, AppError> {
        let artists = sqlx::query_as::<_, ArtistDto>(
            "SELECT id, name FROM artists
             WHERE name LIKE ?
             ORDER BY name
             LIMIT ? OFFSET ?",
        )
        .bind(format!("%{}%", query))
        .bind(count)
        .bind(offset)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok(artists)
    }

    /// 搜索专辑 (详细信息)
    ///
    /// # 参数
    ///
    /// * `query` - 搜索关键词
    /// * `count` - 返回数量
    /// * `offset` - 偏移量
    async fn search_albums_detailed(
        &self,
        query: &str,
        count: i32,
        offset: i32,
    ) -> Result<Vec<AlbumDetailDto>, AppError> {
        let albums = sqlx::query_as::<_, AlbumDetailDto>(
            "SELECT a.id, a.name, ar.name as artist, a.artist_id, a.year, a.genre,
                    a.cover_art_path, a.song_count, a.duration, a.play_count
             FROM albums a
             JOIN artists ar ON a.artist_id = ar.id
             WHERE a.name LIKE ? OR ar.name LIKE ?
             ORDER BY a.name
             LIMIT ? OFFSET ?",
        )
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .bind(count)
        .bind(offset)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok(albums)
    }

    /// 搜索专辑 (简化信息)
    ///
    /// # 参数
    ///
    /// * `query` - 搜索关键词
    /// * `count` - 返回数量
    /// * `offset` - 偏移量
    async fn search_albums_simple(
        &self,
        query: &str,
        count: i32,
        offset: i32,
    ) -> Result<Vec<AlbumDto>, AppError> {
        let albums = sqlx::query_as::<_, AlbumDto>(
            "SELECT a.id, a.name, ar.name as artist, a.year, a.song_count
             FROM albums a
             JOIN artists ar ON a.artist_id = ar.id
             WHERE a.name LIKE ? OR ar.name LIKE ?
             ORDER BY a.name
             LIMIT ? OFFSET ?",
        )
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .bind(count)
        .bind(offset)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok(albums)
    }

    /// 搜索歌曲
    ///
    /// # 参数
    ///
    /// * `query` - 搜索关键词
    /// * `count` - 返回数量
    /// * `offset` - 偏移量
    async fn search_songs(
        &self,
        user_id: &str,
        query: &str,
        count: i32,
        offset: i32,
    ) -> Result<Vec<ComplexSongDto>, AppError> {
        let songs = sqlx::query_as::<_, SongDetailDto>(&format!(
            "{} WHERE s.title LIKE ? OR al.name LIKE ? OR ar.name LIKE ?
             ORDER BY s.title
             LIMIT ? OFFSET ?",
            sql_utils::detail_sql()
        ))
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .bind(count)
        .bind(offset)
        .fetch_all(&self.ctx.pool)
        .await?;

        // 使用 SongService 丰富歌曲信息
        let song_service = SongService::new(self.ctx.clone());
        let complex_songs = song_service.enrich_songs(user_id, songs).await?;

        tracing::info!(
            "limit {}, offset {}, query={} len={}",
            count,
            offset,
            query,
            complex_songs.len()
        );
        Ok(complex_songs)
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
                year INTEGER,
                genre TEXT,
                cover_art_path TEXT,
                song_count INTEGER DEFAULT 0,
                duration INTEGER DEFAULT 0,
                play_count INTEGER DEFAULT 0
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
        sqlx::query("INSERT INTO artists (id, name) VALUES ('artist1', 'Test Artist'), ('artist2', 'Another Artist')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO albums (id, name, artist_id, year, genre, song_count)
             VALUES ('album1', 'Test Album', 'artist1', 2020, 'Rock', 2),
                    ('album2', 'Another Album', 'artist2', 2021, 'Pop', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO songs (id, title, artist_id, album_id, duration, track_number, file_path)
             VALUES ('song1', 'Test Song', 'artist1', 'album1', 180, 1, '/test/song1.mp3'),
                    ('song2', 'Another Song', 'artist2', 'album2', 200, 1, '/test/song2.mp3')",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    fn create_service(pool: SqlitePool) -> SearchService {
        let ctx = Arc::new(ServiceContext::new(pool));
        SearchService::new(ctx)
    }

    #[tokio::test]
    async fn test_search_artists() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let artists = service.search_artists("Test", 10, 0).await.unwrap();
        assert_eq!(artists.len(), 1);
        assert_eq!(artists[0].name, "Test Artist");
    }

    #[tokio::test]
    async fn test_search_albums_detailed() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let albums = service.search_albums_detailed("Test", 10, 0).await.unwrap();
        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0].name, "Test Album");
    }

    #[tokio::test]
    async fn test_search_songs() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let songs = service.search_songs("", "Test", 10, 0).await.unwrap();
        assert_eq!(songs.len(), 1);
        assert_eq!(songs[0].song.title, "Test Song");
    }

    #[tokio::test]
    async fn test_search_all_parallel() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let params = SearchParams {
            query: "Test".to_string(),
            ..Default::default()
        };

        let results = service.search_all("", params).await.unwrap();

        assert_eq!(results.artists.len(), 1);
        assert_eq!(results.albums.len(), 1);
        assert_eq!(results.songs.len(), 1);
    }

    #[tokio::test]
    async fn test_search_all_simple_parallel() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let params = SearchParams {
            query: "Another".to_string(),
            ..Default::default()
        };

        let results = service.search_all_simple("", params).await.unwrap();

        assert_eq!(results.artists.len(), 1);
        assert_eq!(results.albums.len(), 1);
        assert_eq!(results.songs.len(), 1);
    }

    #[tokio::test]
    async fn test_search_pagination() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        // 搜索第一页
        let artists1 = service.search_artists("Artist", 1, 0).await.unwrap();
        assert_eq!(artists1.len(), 1);

        // 搜索第二页
        let artists2 = service.search_artists("Artist", 1, 1).await.unwrap();
        assert_eq!(artists2.len(), 1);

        // 确保不同
        assert_ne!(artists1[0].id, artists2[0].id);
    }
}
