//! 浏览功能服务
//!
//! 负责处理音乐库浏览相关的业务逻辑:
//! - 艺术家/专辑/歌曲查询
//! - 专辑列表(支持多种排序类型)
//! - 随机歌曲
//! - 流派查询
#![allow(dead_code)]

use crate::error::AppError;
use crate::models::dto::{
    AlbumDetailDto, ArtistDetailDto, ArtistDto, ComplexSongDto, SongDetailDto,
};
use crate::services::ServiceContext;
use crate::utils::{image_utils, sql_utils};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

/// 流派信息 (name, song_count, album_count)
pub type GenreInfo = (String, i32, i32);

/// 专辑列表排序类型
#[derive(Debug, Clone, Copy, Default)]
pub enum AlbumListType {
    Random,
    #[default]
    Newest,
    Highest,
    Frequent,
    Recent,
    AlphabeticalByName,
    AlphabeticalByArtist,
    ByYear,
    ByGenre,
}

impl FromStr for AlbumListType {
    type Err = String;

    /// 从字符串解析专辑列表类型
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = match s {
            "random" => AlbumListType::Random,
            "newest" => AlbumListType::Newest,
            "highest" => AlbumListType::Highest,
            "frequent" => AlbumListType::Frequent,
            "recent" => AlbumListType::Recent,
            "alphabetical" | "alphabeticalByName" => AlbumListType::AlphabeticalByName,
            "alphabeticalByArtist" => AlbumListType::AlphabeticalByArtist,
            "byYear" => AlbumListType::ByYear,
            "byGenre" => AlbumListType::ByGenre,
            _ => AlbumListType::Newest, // 默认按最新
        };
        Ok(s)
    }
}

impl AlbumListType {
    /// 获取 ORDER BY 子句
    fn order_by_clause(&self) -> &'static str {
        match self {
            AlbumListType::Random => "RANDOM()",
            AlbumListType::Newest => "a.created_at DESC",
            AlbumListType::Highest => "a.play_count DESC",
            AlbumListType::Frequent => "a.play_count DESC",
            AlbumListType::Recent => "a.updated_at DESC",
            AlbumListType::AlphabeticalByName => "a.name ASC",
            AlbumListType::AlphabeticalByArtist => "ar.name ASC, a.name ASC",
            AlbumListType::ByYear => "a.year DESC, a.name ASC",
            AlbumListType::ByGenre => "a.genre ASC, a.name ASC",
        }
    }

    /// 获取 WHERE 子句 (某些类型需要额外的过滤条件)
    fn where_clause(&self) -> Option<&'static str> {
        match self {
            AlbumListType::ByYear => Some("a.year IS NOT NULL"),
            AlbumListType::ByGenre => Some("a.genre IS NOT NULL"),
            _ => None,
        }
    }
}

/// 浏览功能服务
pub struct BrowsingService {
    ctx: Arc<ServiceContext>,
}

impl BrowsingService {
    /// 创建新的 BrowsingService
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// 获取专辑列表 (统一查询接口)
    ///
    /// # 参数
    ///
    /// * `list_type` - 列表类型 (random/newest/highest等)
    /// * `size` - 返回数量
    /// * `offset` - 偏移量
    ///
    /// # 优化
    ///
    /// 使用查询构建器避免重复的 SQL 模板代码
    pub async fn get_album_list(
        &self,
        list_type: AlbumListType,
        size: i32,
        offset: i32,
    ) -> Result<Vec<AlbumDetailDto>, AppError> {
        // 构建基础查询
        let base_query = "SELECT a.id, a.name, ar.name as artist, a.artist_id, a.year, a.genre,
                    a.cover_art_path, a.song_count, a.duration, a.play_count
             FROM albums a
             JOIN artists ar ON a.artist_id = ar.id";

        // 添加 WHERE 子句 (如果需要)
        let where_clause = list_type.where_clause();
        let order_by = list_type.order_by_clause();

        let query = if let Some(where_cond) = where_clause {
            format!(
                "{} WHERE {} ORDER BY {} LIMIT ? OFFSET ?",
                base_query, where_cond, order_by
            )
        } else {
            format!("{} ORDER BY {} LIMIT ? OFFSET ?", base_query, order_by)
        };

        let albums = sqlx::query_as::<_, AlbumDetailDto>(&query)
            .bind(size)
            .bind(offset)
            .fetch_all(&self.ctx.pool)
            .await?;

        Ok(albums)
    }

    /// 获取随机歌曲
    ///
    /// # 参数
    ///
    /// * `size` - 返回数量
    /// * `genre` - 可选的流派过滤
    /// * `from_year` - 可选的起始年份
    /// * `to_year` - 可选的结束年份
    pub async fn get_random_songs(
        &self,
        size: i32,
        genre: Option<&str>,
        from_year: Option<i32>,
        to_year: Option<i32>,
    ) -> Result<Vec<SongDetailDto>, AppError> {
        let mut conditions = vec!["1=1".to_string()];

        if let Some(g) = genre {
            conditions.push(format!("al.genre = '{}'", g));
        }
        if let Some(from) = from_year {
            conditions.push(format!("al.year >= {}", from));
        }
        if let Some(to) = to_year {
            conditions.push(format!("al.year <= {}", to));
        }

        let query = format!(
            "{} WHERE {}
             ORDER BY RANDOM()
             LIMIT ?",
            sql_utils::detail_sql(),
            conditions.join(" AND ")
        );

        let songs = sqlx::query_as::<_, SongDetailDto>(&query)
            .bind(size)
            .fetch_all(&self.ctx.pool)
            .await?;

        Ok(songs)
    }

    /// 获取艺术家的热门歌曲
    ///
    /// # 参数
    ///
    /// * `artist_name` - 艺术家名称
    /// * `count` - 返回数量
    pub async fn get_top_songs(
        &self,
        artist_name: &str,
        count: i32,
    ) -> Result<Vec<SongDetailDto>, AppError> {
        let songs = sqlx::query_as::<_, SongDetailDto>(&format!(
            "{} WHERE ar.name = ?
             ORDER BY s.play_count DESC
             LIMIT ?",
            sql_utils::detail_sql()
        ))
        .bind(artist_name)
        .bind(count)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok(songs)
    }

    /// 按流派获取歌曲
    ///
    /// # 参数
    ///
    /// * `genre` - 流派名称
    /// * `count` - 返回数量
    /// * `offset` - 偏移量
    pub async fn get_songs_by_genre(
        &self,
        genre: &str,
        count: i32,
        offset: i32,
    ) -> Result<Vec<SongDetailDto>, AppError> {
        let songs = sqlx::query_as::<_, SongDetailDto>(&format!(
            "{} WHERE s.genre = ?
             ORDER BY ar.name ASC, al.name ASC
             LIMIT ? OFFSET ?",
            sql_utils::detail_sql()
        ))
        .bind(genre)
        .bind(count)
        .bind(offset)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok(songs)
    }

    /// 获取所有流派
    pub async fn get_genres(&self) -> Result<Vec<GenreInfo>, AppError> {
        let genres = sqlx::query_as::<_, GenreInfo>(
            "SELECT genre as name, COUNT(*) as song_count, COUNT(DISTINCT album_id) as album_count
             FROM songs
             WHERE genre IS NOT NULL
             GROUP BY genre
             ORDER BY genre ASC",
        )
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok(genres)
    }

    /// 获取所有艺术家 (索引格式)
    ///
    /// # 返回
    ///
    /// 按字母分组的艺术家列表
    pub async fn get_artist_indexes(&self) -> Result<Vec<ArtistDto>, AppError> {
        let artists =
            sqlx::query_as::<_, ArtistDto>("SELECT id, name FROM artists ORDER BY name ASC")
                .fetch_all(&self.ctx.pool)
                .await?;

        Ok(artists)
    }

    /// 获取单个艺术家详情
    ///
    /// # 参数
    ///
    /// * `artist_id` - 艺术家 ID
    pub async fn get_artist(
        &self,
        artist_id: &str,
    ) -> Result<(ArtistDetailDto, Vec<AlbumDetailDto>), AppError> {
        // 获取艺术家信息
        let artist = sqlx::query_as::<_, ArtistDetailDto>(
            "SELECT id, name, cover_art_path FROM artists WHERE id = ?",
        )
        .bind(artist_id)
        .fetch_optional(&self.ctx.pool)
        .await?
        .ok_or_else(|| AppError::not_found("Artist"))?;

        // 获取艺术家的专辑
        let albums = sqlx::query_as::<_, AlbumDetailDto>(
            "SELECT a.id, a.name, ar.name as artist, a.artist_id, a.year, a.genre,
                    a.cover_art_path, a.song_count, a.duration, a.play_count
             FROM albums a
             JOIN artists ar ON a.artist_id = ar.id
             WHERE a.artist_id = ?
             ORDER BY a.year DESC, a.name ASC",
        )
        .bind(artist_id)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok((artist, albums))
    }

    /// 获取单个专辑详情
    ///
    /// # 参数
    ///
    /// * `album_id` - 专辑 ID
    pub async fn get_album(
        &self,
        album_id: &str,
    ) -> Result<(AlbumDetailDto, Vec<SongDetailDto>), AppError> {
        // 获取专辑信息
        let album = sqlx::query_as::<_, AlbumDetailDto>(
            "SELECT a.id, a.name, ar.name as artist, a.artist_id, a.year, a.genre,
                    a.cover_art_path, a.song_count, a.duration, a.play_count
             FROM albums a
             JOIN artists ar ON a.artist_id = ar.id
             WHERE a.id = ?",
        )
        .bind(album_id)
        .fetch_optional(&self.ctx.pool)
        .await?
        .ok_or_else(|| AppError::not_found("Album"))?;

        // 获取专辑的歌曲
        let songs = sqlx::query_as::<_, SongDetailDto>(&format!(
            "{}
             WHERE s.album_id = ?
             ORDER BY s.disc_number ASC, s.track_number ASC",
            sql_utils::detail_sql()
        ))
        .bind(album_id)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok((album, songs))
    }

    /// 获取单个歌曲详情
    ///
    /// # 参数
    ///
    /// * `song_id` - 歌曲 ID
    pub async fn get_song(&self, user_id: &str, song_id: &str) -> Result<ComplexSongDto, AppError> {
        let song = sqlx::query_as::<_, SongDetailDto>(&format!(
            "{} WHERE s.id = ?",
            sql_utils::detail_sql()
        ))
        .bind(song_id)
        .fetch_optional(&self.ctx.pool)
        .await?
        .ok_or_else(|| AppError::not_found("Song"))?;

        let suffix = if song.path.is_some() {
            Some(image_utils::get_content_type(Path::new(
                &song.path.clone().unwrap(),
            )))
        } else {
            None
        };

        let rating = sqlx::query_scalar::<_, i32>(
            "SELECT rating FROM ratings WHERE user_id = ? AND song_id = ?",
        )
        .bind(user_id)
        .bind(&song.id)
        .fetch_optional(&self.ctx.pool)
        .await?;

        let complex_song = ComplexSongDto {
            song,
            user_rating: rating,
            is_starred: None,
            suffix,
        };

        Ok(complex_song)
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
                play_count INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
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
                track INTEGER,
                disc_number INTEGER,
                size INTEGER,
                suffix TEXT,
                content_type TEXT,
                path TEXT,
                play_count INTEGER DEFAULT 0,
                genre TEXT
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

        sqlx::query(
            "INSERT INTO albums (id, name, artist_id, year, genre, song_count, duration, play_count)
             VALUES ('album1', 'Test Album', 'artist1', 2020, 'Rock', 2, 300, 100)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO albums (id, name, artist_id, year, genre, song_count, duration, play_count)
             VALUES ('album2', 'Another Album', 'artist1', 2021, 'Pop', 1, 200, 50)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO songs (id, title, artist_id, album_id, duration, track, genre, play_count)
             VALUES ('song1', 'Song 1', 'artist1', 'album1', 180, 1, 'Rock', 50)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO songs (id, title, artist_id, album_id, duration, track, genre, play_count)
             VALUES ('song2', 'Song 2', 'artist1', 'album1', 120, 2, 'Rock', 30)",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    fn create_service(pool: SqlitePool) -> BrowsingService {
        let ctx = Arc::new(ServiceContext::new(pool));
        BrowsingService::new(ctx)
    }

    #[tokio::test]
    async fn test_get_album_list_newest() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let albums = service
            .get_album_list(AlbumListType::Newest, 10, 0)
            .await
            .unwrap();

        assert_eq!(albums.len(), 2);
        // 最新的应该在前面 (按 created_at DESC)
        assert_eq!(albums[0].name, "Another Album");
    }

    #[tokio::test]
    async fn test_get_album_list_highest() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let albums = service
            .get_album_list(AlbumListType::Highest, 10, 0)
            .await
            .unwrap();

        assert_eq!(albums.len(), 2);
        // 播放次数高的在前面
        assert_eq!(albums[0].name, "Test Album");
        assert_eq!(albums[0].play_count, 100);
    }

    #[tokio::test]
    async fn test_get_album_list_pagination() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let albums = service
            .get_album_list(AlbumListType::Newest, 1, 0)
            .await
            .unwrap();
        assert_eq!(albums.len(), 1);

        let albums = service
            .get_album_list(AlbumListType::Newest, 1, 1)
            .await
            .unwrap();
        assert_eq!(albums.len(), 1);
    }

    #[tokio::test]
    async fn test_get_top_songs() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let songs = service.get_top_songs("Test Artist", 10).await.unwrap();

        assert_eq!(songs.len(), 2);
        // 播放次数高的在前面
        assert_eq!(songs[0].title, "Song 1");
        // assert_eq!(songs[0].play_count, 50);
    }

    #[tokio::test]
    async fn test_get_artist() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let (artist, albums) = service.get_artist("artist1").await.unwrap();

        assert_eq!(artist.name, "Test Artist");
        assert_eq!(albums.len(), 2);
    }

    #[tokio::test]
    async fn test_get_album() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let (album, songs) = service.get_album("album1").await.unwrap();

        assert_eq!(album.name, "Test Album");
        assert_eq!(songs.len(), 2);
    }

    #[tokio::test]
    async fn test_get_song() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let song = service.get_song("1", "song1").await.unwrap();

        assert_eq!(song.song.title, "Song 1");
        assert_eq!(song.song.artist, "Test Artist");
    }

    #[tokio::test]
    async fn test_get_genres() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let genres = service.get_genres().await.unwrap();

        println!("genres: {:?}", genres);

        assert!(genres.len() >= 1);
        // 应该有 Rock 流派
        // assert!(genres.iter().any(|g| g.name == "Rock"));
    }
}
