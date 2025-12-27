//! 音乐库扫描服务
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use sqlx::SqlitePool;
use crate::error::AppError;
use crate::models::artist::Artist;
use crate::models::album::Album;
use crate::models::song::Song;

/// 音乐库扫描服务
pub struct ScanService {
    pool: SqlitePool,
    library_path: PathBuf,
}

/// 扫描结果
#[derive(Debug, Default)]
pub struct ScanResult {
    pub artists: usize,
    pub albums: usize,
    pub songs: usize,
    pub failed: usize,
}

impl ScanService {
    pub fn new(pool: SqlitePool, library_path: PathBuf) -> Self {
        Self { pool, library_path }
    }

    /// 扫描音乐库
    pub async fn scan_library(&self) -> Result<ScanResult, AppError> {
        let mut result = ScanResult::default();

        if !self.library_path.exists() {
            return Err(AppError::not_found("Music library path"));
        }

        tracing::info!("开始扫描音乐库: {:?}", self.library_path);

        for entry in WalkDir::new(&self.library_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

            // 支持的音频格式
            if matches!(ext.as_str(), "mp3" | "flac" | "wav" | "m4a" | "aac" | "ogg" | "opus") {
                match self.process_audio_file(path).await {
                    Ok(_) => {
                        result.songs += 1;
                    }
                    Err(e) => {
                        tracing::warn!("处理文件失败 {}: {}", path.display(), e);
                        result.failed += 1;
                    }
                }
            }
        }

        // 更新艺术家和专辑计数
        let artist_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM artists")
            .fetch_one(&self.pool)
            .await? as usize;
        let album_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM albums")
            .fetch_one(&self.pool)
            .await? as usize;

        result.artists = artist_count;
        result.albums = album_count;

        tracing::info!("音乐库扫描完成: {:?}", result);
        Ok(result)
    }

    /// 处理单个音频文件
    async fn process_audio_file(&self, path: &Path) -> Result<(), AppError> {
        // 简化处理：从文件名提取基本信息
        // 实际应用中可以使用 symphonia 或其他音频库来读取完整的元数据

        // 从文件名提取标题
        let title = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // 从父目录提取艺术家和专辑信息
        let artist_name = path.parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Artist")
            .to_string();

        let album_name = path.parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Album")
            .to_string();

        // 获取文件大小
        let file_size = std::fs::metadata(path)
            .map(|m| m.len())
            .ok();

        // 简化的元数据
        let duration = 0; // 需要音频库来读取
        let bit_rate: Option<i32> = None;
        let year: Option<i32> = None;
        let genre: Option<String> = None;
        let track_number: Option<i32> = None;
        let disc_number: Option<i32> = None;
        let content_type = "audio/mpeg".to_string(); // 简化处理

        // 保存到数据库
        self.save_to_database(
            &artist_name,
            &album_name,
            &title,
            path,
            duration,
            bit_rate,
            year,
            genre.as_deref(),
            track_number,
            disc_number,
            &content_type,
            file_size,
        ).await?;

        Ok(())
    }

    /// 保存到数据库
    #[allow(clippy::too_many_arguments)]
    async fn save_to_database(
        &self,
        artist_name: &str,
        album_name: &str,
        title: &str,
        path: &Path,
        duration: i32,
        bit_rate: Option<i32>,
        year: Option<i32>,
        genre: Option<&str>,
        track_number: Option<i32>,
        disc_number: Option<i32>,
        content_type: &str,
        file_size: Option<u64>,
    ) -> Result<(), AppError> {
        // 插入或更新艺术家
        let artist_id = self.get_or_create_artist(artist_name).await?;

        // 插入或更新专辑
        let album_id = self.get_or_create_album(&artist_id, album_name, year, genre).await?;

        // 插入或更新歌曲
        self.get_or_create_song(
            &album_id,
            &artist_id,
            title,
            duration,
            bit_rate,
            year,
            genre,
            track_number,
            disc_number,
            path,
            content_type,
            file_size,
        ).await?;

        Ok(())
    }

    /// 获取或创建艺术家
    async fn get_or_create_artist(&self, name: &str) -> Result<String, AppError> {
        // 先尝试查找
        let existing = sqlx::query_scalar::<_, String>(
            "SELECT id FROM artists WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(id) = existing {
            return Ok(id);
        }

        // 创建新艺术家
        let artist = Artist::new(name.to_string(), None, None);
        sqlx::query(
            "INSERT INTO artists (id, name, music_brainz_id, cover_art_path, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&artist.id)
        .bind(&artist.name)
        .bind(&artist.music_brainz_id)
        .bind(&artist.cover_art_path)
        .bind(artist.created_at)
        .bind(artist.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(artist.id)
    }

    /// 获取或创建专辑
    async fn get_or_create_album(
        &self,
        artist_id: &str,
        name: &str,
        year: Option<i32>,
        genre: Option<&str>,
    ) -> Result<String, AppError> {
        // 先尝试查找
        let existing = sqlx::query_scalar::<_, String>(
            "SELECT id FROM albums WHERE artist_id = ? AND name = ?"
        )
        .bind(artist_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(id) = existing {
            return Ok(id);
        }

        // 创建新专辑
        let album = Album::new(
            artist_id.to_string(),
            name.to_string(),
            path_to_string(&self.library_path), // 简化处理
            year,
            genre.map(|s| s.to_string()),
            None,
        );
        sqlx::query(
            "INSERT INTO albums (id, artist_id, name, year, genre, cover_art_path, path,
             song_count, duration, play_count, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0, 0, ?, ?)"
        )
        .bind(&album.id)
        .bind(&album.artist_id)
        .bind(&album.name)
        .bind(&album.year)
        .bind(&album.genre)
        .bind(&album.cover_art_path)
        .bind(&album.path)
        .bind(album.created_at)
        .bind(album.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(album.id)
    }

    /// 获取或创建歌曲
    #[allow(clippy::too_many_arguments)]
    async fn get_or_create_song(
        &self,
        album_id: &str,
        artist_id: &str,
        title: &str,
        duration: i32,
        bit_rate: Option<i32>,
        year: Option<i32>,
        genre: Option<&str>,
        track_number: Option<i32>,
        disc_number: Option<i32>,
        path: &Path,
        content_type: &str,
        file_size: Option<u64>,
    ) -> Result<(), AppError> {
        // 先尝试查找
        let existing = sqlx::query_scalar::<_, String>(
            "SELECT id FROM songs WHERE album_id = ? AND title = ? AND file_path = ?"
        )
        .bind(album_id)
        .bind(title)
        .bind(path_to_string(path))
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            return Ok(());
        }

        // 创建新歌曲
        let song = Song::new(
            album_id.to_string(),
            artist_id.to_string(),
            title.to_string(),
            duration,
            path_to_string(path),
            track_number,
            disc_number,
            bit_rate,
            genre.map(|s| s.to_string()),
            year,
            Some(content_type.to_string()),
            file_size.map(|s| s as i64),
        );

        sqlx::query(
            "INSERT INTO songs (id, album_id, artist_id, title, track_number, disc_number,
             duration, bit_rate, genre, year, content_type, file_path, file_size, play_count,
             created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?)"
        )
        .bind(&song.id)
        .bind(&song.album_id)
        .bind(&song.artist_id)
        .bind(&song.title)
        .bind(&song.track_number)
        .bind(&song.disc_number)
        .bind(&song.duration)
        .bind(&song.bit_rate)
        .bind(&song.genre)
        .bind(&song.year)
        .bind(&song.content_type)
        .bind(&song.file_path)
        .bind(&song.file_size)
        .bind(song.created_at)
        .bind(song.updated_at)
        .execute(&self.pool)
        .await?;

        // 更新专辑的歌曲计数
        sqlx::query(
            "UPDATE albums
             SET song_count = (SELECT COUNT(*) FROM songs WHERE album_id = ?),
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = ?"
        )
        .bind(album_id)
        .bind(album_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
