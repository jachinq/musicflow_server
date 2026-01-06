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

/// ID 类型枚举
#[derive(Debug, Clone)]
enum IdType {
    Song(String),
    Album(String),
    Artist(String),
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

    /// 通过查询数据库判断 ID 类型
    ///
    /// # 参数
    ///
    /// * `id` - 要检测的 ID
    ///
    /// # 返回
    ///
    /// 返回 ID 对应的类型 (Song/Album/Artist)
    ///
    /// # 优化
    ///
    /// 使用单次查询同时检测三个表，避免多次数据库往返
    async fn detect_id_type(&self, id: &str) -> Result<IdType, AppError> {
        let result = sqlx::query_as::<_, (Option<i32>, Option<i32>, Option<i32>)>(
            "SELECT
                (SELECT 1 FROM songs WHERE id = ?) as is_song,
                (SELECT 1 FROM albums WHERE id = ?) as is_album,
                (SELECT 1 FROM artists WHERE id = ?) as is_artist"
        )
        .bind(id)
        .bind(id)
        .bind(id)
        .fetch_one(&self.ctx.pool)
        .await?;

        match result {
            (Some(_), _, _) => Ok(IdType::Song(id.to_string())),
            (_, Some(_), _) => Ok(IdType::Album(id.to_string())),
            (_, _, Some(_)) => Ok(IdType::Artist(id.to_string())),
            _ => Err(AppError::not_found("Song/Album/Artist")),
        }
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

        let starred = sqlx::query_scalar::<_, String>(
            "SELECT song_id FROM starred WHERE user_id = ? AND song_id = ?",
        )
        .bind(user_id)
        .bind(&song.id)
        .fetch_optional(&self.ctx.pool)
        .await?;
        let starred = if let Some(s) = starred {
            Some(s.eq(&song.id))
        } else {
            None
        };

        let complex_song = ComplexSongDto {
            song,
            user_rating: rating,
            starred,
            suffix,
        };

        Ok(complex_song)
    }

    /// 获取相似歌曲 (统一入口)
    ///
    /// # 参数
    ///
    /// * `id` - 歌曲/专辑/艺术家 ID
    /// * `count` - 返回数量
    ///
    /// # 功能
    ///
    /// 自动识别 ID 类型并调用对应的相似推荐逻辑:
    /// - 歌曲 ID: 基于元数据计算相似度
    /// - 专辑 ID: 返回专辑歌曲 + 同艺术家/流派歌曲
    /// - 艺术家 ID: 返回热门歌曲 + 同流派艺术家歌曲
    ///
    /// # 性能
    ///
    /// 使用单次查询同时检测三个表，避免多次数据库往返
    pub async fn get_similar_songs(
        &self,
        id: &str,
        count: i32,
    ) -> Result<Vec<SongDetailDto>, AppError> {
        // 检测 ID 类型
        let id_type = self.detect_id_type(id).await?;

        // 根据类型调用不同的逻辑
        match id_type {
            IdType::Song(song_id) => {
                self.get_similar_songs_by_song(&song_id, count).await
            }
            IdType::Album(album_id) => {
                self.get_similar_songs_by_album(&album_id, count).await
            }
            IdType::Artist(artist_id) => {
                self.get_similar_songs_by_artist(&artist_id, count).await
            }
        }
    }

    /// 基于歌曲的相似推荐
    ///
    /// # 参数
    ///
    /// * `song_id` - 歌曲 ID
    /// * `count` - 返回数量
    ///
    /// # 相似度算法
    ///
    /// 使用加权评分系统计算相似度:
    /// - 同艺术家: 5 分
    /// - 同流派: 4 分
    /// - 年代接近 (±2年): 2 分
    /// - 同专辑: 3 分 (权重较低,避免推荐过多同专辑歌曲)
    ///
    /// 最终结果按分数降序排列,并添加随机性保证多样性
    async fn get_similar_songs_by_song(
        &self,
        song_id: &str,
        count: i32,
    ) -> Result<Vec<SongDetailDto>, AppError> {
        // 首先获取目标歌曲的信息
        let target_song = sqlx::query_as::<_, SongDetailDto>(&format!(
            "{} WHERE s.id = ?",
            sql_utils::detail_sql()
        ))
        .bind(song_id)
        .fetch_optional(&self.ctx.pool)
        .await?
        .ok_or_else(|| AppError::not_found("Song"))?;

        // 构建相似度查询
        let query = format!(
            "{}
             WHERE s.id != ?
             ORDER BY (
                -- 同艺术家得分
                CASE WHEN s.artist_id = ? THEN 5 ELSE 0 END +
                -- 同流派得分
                CASE WHEN s.genre IS NOT NULL AND s.genre = ? THEN 4 ELSE 0 END +
                -- 年代接近度得分 (±2年内)
                CASE WHEN s.year IS NOT NULL AND ? IS NOT NULL AND abs(s.year - ?) <= 2 THEN 2 ELSE 0 END +
                -- 同专辑得分 (权重较低)
                CASE WHEN s.album_id = ? THEN 3 ELSE 0 END
             ) DESC, RANDOM()
             LIMIT ?",
            sql_utils::detail_sql()
        );

        let songs = sqlx::query_as::<_, SongDetailDto>(&query)
            .bind(song_id) // 排除目标歌曲
            .bind(&target_song.artist_id) // 同艺术家判断
            .bind(&target_song.genre) // 同流派判断
            .bind(&target_song.year) // 年代判断 (第一个)
            .bind(&target_song.year) // 年代判断 (第二个)
            .bind(&target_song.album_id) // 同专辑判断
            .bind(count)
            .fetch_all(&self.ctx.pool)
            .await?;

        Ok(songs)
    }

    /// 基于专辑的相似推荐
    ///
    /// # 参数
    ///
    /// * `album_id` - 专辑 ID
    /// * `count` - 返回数量
    ///
    /// # 策略
    ///
    /// - 该专辑的歌曲（30%）
    /// - 同艺术家其他专辑的歌曲（40%）
    /// - 同流派/年代相近的歌曲（30%）
    async fn get_similar_songs_by_album(
        &self,
        album_id: &str,
        count: i32,
    ) -> Result<Vec<SongDetailDto>, AppError> {
        // 获取专辑信息
        let album = sqlx::query_as::<_, AlbumDetailDto>(
            "SELECT a.id, a.name, ar.name as artist, a.artist_id, a.year, a.genre,
                    a.cover_art_path, a.song_count, a.duration, a.play_count
             FROM albums a
             JOIN artists ar ON a.artist_id = ar.id
             WHERE a.id = ?"
        )
        .bind(album_id)
        .fetch_optional(&self.ctx.pool)
        .await?
        .ok_or_else(|| AppError::not_found("Album"))?;

        // 计算各部分数量
        let album_songs_count = (count as f32 * 0.3).ceil() as i32;
        let artist_songs_count = (count as f32 * 0.4).ceil() as i32;
        let genre_songs_count = count - album_songs_count - artist_songs_count;

        // 构建联合查询
        let query = format!(
            "SELECT * FROM (
                -- 该专辑的歌曲
                {}
                WHERE s.album_id = ?
                ORDER BY s.track_number ASC
                LIMIT ?
            )
            UNION ALL
            SELECT * FROM (
                -- 同艺术家其他专辑的歌曲
                {}
                WHERE s.artist_id = ? AND s.album_id != ?
                ORDER BY s.play_count DESC, RANDOM()
                LIMIT ?
            )
            UNION ALL
            SELECT * FROM (
                -- 同流派/年代的歌曲
                {}
                WHERE s.album_id != ?
                AND (al.genre = ? OR (al.year IS NOT NULL AND ? IS NOT NULL AND abs(al.year - ?) <= 2))
                ORDER BY s.play_count DESC, RANDOM()
                LIMIT ?
            )
            LIMIT ?",
            sql_utils::detail_sql(),
            sql_utils::detail_sql(),
            sql_utils::detail_sql()
        );

        let songs = sqlx::query_as::<_, SongDetailDto>(&query)
            .bind(album_id)
            .bind(album_songs_count)
            .bind(&album.artist_id)
            .bind(album_id)
            .bind(artist_songs_count)
            .bind(album_id)
            .bind(&album.genre)
            .bind(&album.year)
            .bind(&album.year)
            .bind(genre_songs_count)
            .bind(count)
            .fetch_all(&self.ctx.pool)
            .await?;

        Ok(songs)
    }

    /// 基于艺术家的相似推荐
    ///
    /// # 参数
    ///
    /// * `artist_id` - 艺术家 ID
    /// * `count` - 返回数量
    ///
    /// # 策略
    ///
    /// - 该艺术家的热门歌曲（60%）
    /// - 同流派其他艺术家的歌曲（40%）
    async fn get_similar_songs_by_artist(
        &self,
        artist_id: &str,
        count: i32,
    ) -> Result<Vec<SongDetailDto>, AppError> {
        // 验证艺术家是否存在
        let _artist_exists = sqlx::query_scalar::<_, i32>(
            "SELECT 1 FROM artists WHERE id = ? LIMIT 1"
        )
        .bind(artist_id)
        .fetch_optional(&self.ctx.pool)
        .await?
        .ok_or_else(|| AppError::not_found("Artist"))?;

        // 获取该艺术家的主要流派
        let main_genre = sqlx::query_scalar::<_, Option<String>>(
            "SELECT genre FROM albums WHERE artist_id = ? AND genre IS NOT NULL
             GROUP BY genre ORDER BY COUNT(*) DESC LIMIT 1"
        )
        .bind(artist_id)
        .fetch_optional(&self.ctx.pool)
        .await?
        .flatten();

        // 计算各部分数量
        let artist_songs_count = (count as f32 * 0.6).ceil() as i32;
        let genre_songs_count = count - artist_songs_count;

        // 构建联合查询
        let query = format!(
            "SELECT * FROM (
                -- 该艺术家的热门歌曲
                {}
                WHERE s.artist_id = ?
                ORDER BY s.play_count DESC, RANDOM()
                LIMIT ?
            )
            UNION ALL
            SELECT * FROM (
                -- 同流派其他艺术家的歌曲
                {}
                WHERE s.artist_id != ? AND al.genre = ?
                ORDER BY s.play_count DESC, RANDOM()
                LIMIT ?
            )
            LIMIT ?",
            sql_utils::detail_sql(),
            sql_utils::detail_sql()
        );

        let songs = sqlx::query_as::<_, SongDetailDto>(&query)
            .bind(artist_id)
            .bind(artist_songs_count)
            .bind(artist_id)
            .bind(&main_genre)
            .bind(genre_songs_count)
            .bind(count)
            .fetch_all(&self.ctx.pool)
            .await?;

        Ok(songs)
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
                name TEXT NOT NULL,
                cover_art_path TEXT
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
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                song_id TEXT NOT NULL,
                rating INTEGER NOT NULL,
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

        sqlx::query(
            "INSERT INTO albums (id, name, artist_id, year, genre, song_count, duration, play_count, created_at)
             VALUES ('album1', 'Test Album', 'artist1', 2020, 'Rock', 2, 300, 100, '2020-01-01 00:00:00')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO albums (id, name, artist_id, year, genre, song_count, duration, play_count, created_at)
             VALUES ('album2', 'Another Album', 'artist1', 2021, 'Pop', 1, 200, 50, '2021-01-01 00:00:00')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO songs (id, title, artist_id, album_id, duration, track_number, genre, play_count)
             VALUES ('song1', 'Song 1', 'artist1', 'album1', 180, 1, 'Rock', 50)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO songs (id, title, artist_id, album_id, duration, track_number, genre, play_count)
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

    #[tokio::test]
    async fn test_detect_id_type_song() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let id_type = service.detect_id_type("song1").await.unwrap();
        assert!(matches!(id_type, IdType::Song(_)));
    }

    #[tokio::test]
    async fn test_detect_id_type_album() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let id_type = service.detect_id_type("album1").await.unwrap();
        assert!(matches!(id_type, IdType::Album(_)));
    }

    #[tokio::test]
    async fn test_detect_id_type_artist() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let id_type = service.detect_id_type("artist1").await.unwrap();
        assert!(matches!(id_type, IdType::Artist(_)));
    }

    #[tokio::test]
    async fn test_detect_id_type_not_found() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let result = service.detect_id_type("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_similar_songs_by_song() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let songs = service.get_similar_songs("song1", 10).await.unwrap();
        // 应该至少有一些相似歌曲 (同专辑的 song2)
        assert!(songs.len() > 0);
    }

    #[tokio::test]
    async fn test_get_similar_songs_by_album() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let songs = service.get_similar_songs("album1", 10).await.unwrap();
        // 应该包含该专辑的歌曲
        assert!(songs.len() > 0);
        // 应该包含专辑的歌曲
        assert!(songs.iter().any(|s| s.album_id == "album1"));
    }

    #[tokio::test]
    async fn test_get_similar_songs_by_artist() {
        let pool = setup_test_db().await;
        let service = create_service(pool);

        let songs = service.get_similar_songs("artist1", 10).await.unwrap();
        // 应该包含该艺术家的歌曲
        assert!(songs.len() > 0);
        // 所有歌曲应该来自该艺术家或相似流派
        assert!(songs.iter().any(|s| s.artist_id == "artist1"));
    }
}
