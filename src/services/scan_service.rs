//! 音乐库扫描服务
#![allow(dead_code)]

use crate::error::AppError;
use crate::models::entities::{Album, Artist, Song};
use crate::utils::{get_image_format, write_image_to_file};
use sha2::{Digest, Sha256};
use sqlx::{Execute, SqlitePool};
use std::fs::File;
use std::path::{Path, PathBuf};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;
use uuid::Uuid;
use walkdir::WalkDir;

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
    pub deleted: usize,
}

/// 音频元数据
#[derive(Debug, Default)]
struct AudioMetadata {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    album_artist: Option<String>,
    genre: Option<String>,
    year: Option<i32>,
    track_number: Option<i32>,
    disc_number: Option<i32>,
    duration_secs: u64,
    bit_rate: Option<i32>,
    sample_rate: Option<i32>,
    channels: Option<u8>,
    content_type: String,
    file_size: Option<u64>,
    cover_art_raw: Option<(String, Box<[u8]>)>,
    lyrics: Option<String>,
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
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            // 支持的音频格式
            if matches!(
                ext.as_str(),
                "mp3" | "flac" | "wav" | "m4a" | "aac" | "ogg" | "opus"
            ) {
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

        // 清理已删除的文件
        let deleted = self.cleanup_deleted_files().await?;
        result.deleted = deleted;

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
        let metadata: AudioMetadata = self.read_audio_metadata(path).await?;
        // 保存到数据库
        self.save_to_database(
            &metadata.artist.as_deref().unwrap_or("Unknown"),
            &metadata.album.as_deref().unwrap_or("Unknown"),
            &metadata.title.as_deref().unwrap_or("Unknown"),
            path,
            metadata.duration_secs as i32,
            metadata.bit_rate,
            metadata.year,
            metadata.genre.as_deref(),
            metadata.track_number,
            metadata.disc_number,
            &metadata.content_type,
            metadata.file_size,
            metadata.cover_art_raw,
            metadata.lyrics.as_deref(),
        )
        .await?;
        Ok(())
    }

    /// 读取单个音频文件元数据
    async fn read_audio_metadata(&self, path: &Path) -> Result<AudioMetadata, AppError> {
        // 使用 Symphonia 读取音频元数据
        let metadata = self.extract_audio_metadata(path)?;

        // 使用元数据中的信息,如果不存在则使用文件路径推断
        let artist_name = metadata
            .artist
            .or_else(|| metadata.album_artist.clone())
            .unwrap_or_else(|| self.extract_artist_from_path(path));

        let album_name = metadata
            .album
            .unwrap_or_else(|| self.extract_album_from_path(path));

        let title = metadata.title.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });

        // 获取文件大小和 MIME 类型
        let file_size = std::fs::metadata(path).map(|m| m.len()).ok();

        let content_type = self.get_content_type(path);

        // 估算比特率:如果有文件大小和时长
        let bit_rate = if let Some(size) = file_size {
            if metadata.duration_secs > 0 {
                Some(((size * 8) / metadata.duration_secs) as i32)
            } else {
                metadata.bit_rate
            }
        } else {
            metadata.bit_rate
        };

        Ok(AudioMetadata {
            title: Some(title),
            artist: Some(artist_name),
            album: Some(album_name),
            album_artist: metadata.album_artist,
            genre: metadata.genre,
            year: metadata.year,
            track_number: metadata.track_number,
            disc_number: metadata.disc_number,
            duration_secs: metadata.duration_secs,
            bit_rate,
            sample_rate: metadata.sample_rate,
            channels: metadata.channels,
            content_type,
            file_size,
            cover_art_raw: metadata.cover_art_raw,
            lyrics: metadata.lyrics,
        })
    }

    /// 使用 Symphonia 提取音频元数据
    fn extract_audio_metadata(&self, path: &Path) -> Result<AudioMetadata, AppError> {
        // 打开文件
        let file = File::open(path)
            .map_err(|e| AppError::ValidationError(format!("无法打开文件: {}", e)))?;

        // 创建媒体源
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // 创建格式提示
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            hint.with_extension(ext);
        }

        // 探测格式
        let mut probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| AppError::ValidationError(format!("无法探测音频格式: {}", e)))?;

        let mut format = probed.format;
        let track = format
            .default_track()
            .ok_or_else(|| AppError::ValidationError("没有找到音频轨道".to_string()))?;

        let mut metadata = AudioMetadata::default();

        // 获取时长信息
        if let Some(time_base) = track.codec_params.time_base {
            if let Some(n_frames) = track.codec_params.n_frames {
                let duration = time_base.calc_time(n_frames);
                metadata.duration_secs = duration.seconds;
            }
        }

        // 获取采样率
        if let Some(sample_rate) = track.codec_params.sample_rate {
            metadata.sample_rate = Some(sample_rate as i32);
        }

        // 获取声道数
        if let Some(channels) = track.codec_params.channels {
            metadata.channels = Some(channels.count() as u8);
        }

        // 比特率计算:如果有文件大小和时长,可以估算
        // Symphonia 的 CodecParameters 不直接提供比特率字段
        // 可以通过 (文件大小 * 8) / 时长 来估算

        // 读取标签元数据
        let meta = if format.metadata().current().is_some() {
            Some(format.metadata())
        } else {
            if probed.metadata.get().is_some() {
                Some(probed.metadata.get().unwrap())
            } else {
                None
            }
        };

        if let Some(meta) = meta {
            if let Some(metadata_rev) = meta.current() {
                for tag in metadata_rev.tags() {
                    match tag.std_key {
                        Some(StandardTagKey::TrackTitle) => {
                            metadata.title = Some(tag.value.to_string());
                        }
                        Some(StandardTagKey::Artist) => {
                            metadata.artist = Some(tag.value.to_string());
                        }
                        Some(StandardTagKey::Album) => {
                            metadata.album = Some(tag.value.to_string());
                        }
                        Some(StandardTagKey::AlbumArtist) => {
                            metadata.album_artist = Some(tag.value.to_string());
                        }
                        Some(StandardTagKey::Genre) => {
                            metadata.genre = Some(tag.value.to_string());
                        }
                        Some(StandardTagKey::Date) | Some(StandardTagKey::ReleaseDate) => {
                            // 尝试解析年份
                            if let Ok(year) = tag.value.to_string().parse::<i32>() {
                                metadata.year = Some(year);
                            } else {
                                // 尝试从日期字符串提取年份 (如 "2024-01-01")
                                if let Some(year_str) = tag.value.to_string().split('-').next() {
                                    if let Ok(year) = year_str.parse::<i32>() {
                                        metadata.year = Some(year);
                                    }
                                }
                            }
                        }
                        Some(StandardTagKey::TrackNumber) => {
                            // 处理 "1/12" 格式
                            let track_str = tag.value.to_string();
                            if let Some(track_num) = track_str.split('/').next() {
                                if let Ok(num) = track_num.parse::<i32>() {
                                    metadata.track_number = Some(num);
                                }
                            }
                        }
                        Some(StandardTagKey::DiscNumber) => {
                            // 处理 "1/2" 格式
                            let disc_str = tag.value.to_string();
                            if let Some(disc_num) = disc_str.split('/').next() {
                                if let Ok(num) = disc_num.parse::<i32>() {
                                    metadata.disc_number = Some(num);
                                }
                            }
                        }
                        Some(StandardTagKey::Lyrics) => {
                            metadata.lyrics = Some(tag.value.to_string());
                        }
                        _ => {}
                    }
                }

                // 处理图片元数据
                let album = metadata_rev
                    .visuals()
                    .iter()
                    .map(|f| (f.media_type.clone(), f.data.clone()))
                    .collect::<Vec<_>>();

                // println!("metadata_rev: {:?}", album.len());
                album.iter().for_each(|f| {
                    metadata.cover_art_raw = Some((f.0.to_string(), f.1.clone()));
                });
            }
        }

        Ok(metadata)
    }

    /// 从路径推断艺术家名称
    fn extract_artist_from_path(&self, path: &Path) -> String {
        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Artist")
            .to_string()
    }

    /// 从路径推断专辑名称
    fn extract_album_from_path(&self, path: &Path) -> String {
        path.parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Album")
            .to_string()
    }

    /// 根据文件扩展名获取 MIME 类型
    fn get_content_type(&self, path: &Path) -> String {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "mp3" => "audio/mpeg",
            "flac" => "audio/flac",
            "wav" => "audio/wav",
            "m4a" | "mp4" => "audio/mp4",
            "aac" => "audio/aac",
            "ogg" => "audio/ogg",
            "opus" => "audio/opus",
            _ => "audio/mpeg",
        }
        .to_string()
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
        cover_art_raw: Option<(String, Box<[u8]>)>,
        lyrics: Option<&str>,
    ) -> Result<(), AppError> {
        // 插入或更新艺术家
        let artist_id = self.get_or_create_artist(artist_name).await?;

        // 插入或更新专辑
        let album_id = self
            .get_or_create_album(&artist_id, album_name, year, genre, cover_art_raw)
            .await?;

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
            lyrics,
        )
        .await?;

        Ok(())
    }

    /// 获取或创建艺术家
    async fn get_or_create_artist(&self, name: &str) -> Result<String, AppError> {
        // 先尝试查找
        let existing = sqlx::query_scalar::<_, String>("SELECT id FROM artists WHERE name = ?")
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
        cover_art_raw: Option<(String, Box<[u8]>)>,
    ) -> Result<String, AppError> {
        // 查找已存在的专辑，获取完整信息用于对比
        let existing = sqlx::query_as::<_, (String, Option<i32>, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, year, genre, cover_art_path, cover_art_hash FROM albums WHERE artist_id = ? AND name = ?",
        )
        .bind(artist_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((
            album_id,
            existing_year,
            existing_genre,
            existing_cover_path,
            existing_cover_hash,
        )) = existing
        {
            // 专辑已存在，进行智能更新
            self.update_album_if_needed(
                &album_id,
                year,
                genre,
                existing_year,
                existing_genre.as_deref(),
                existing_cover_path,
                existing_cover_hash,
                cover_art_raw,
            )
            .await?;
            return Ok(album_id);
        }

        // 专辑不存在，创建新专辑
        self.create_new_album(artist_id, name, year, genre, cover_art_raw)
            .await
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
        lyrics: Option<&str>,
    ) -> Result<(), AppError> {
        // 先尝试查找
        let existing = sqlx::query_scalar::<_, String>("SELECT id FROM songs WHERE file_path = ?")
            .bind(path_to_string(path))
            .fetch_optional(&self.pool)
            .await?;

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
            lyrics.map(|s| s.to_string()),
        );

        if existing.is_some() {
            // 歌曲已存在，进行更新
            let query = sqlx::query(
                "UPDATE songs
                 SET title = ?,
                     track_number = ?,
                     disc_number = ?,
                     duration = ?,
                     bit_rate = ?,
                     genre = ?,
                     year = ?,
                     content_type = ?,
                     file_size = ?,
                     lyrics = ?,
                     updated_at = ?
                 WHERE file_path = ?",
            )
            .bind(&song.title)
            .bind(song.track_number.unwrap_or_default())
            .bind(song.disc_number.unwrap_or_default())
            .bind(song.duration)
            .bind(song.bit_rate.unwrap_or_default())
            .bind(song.genre.clone().unwrap_or_default())
            .bind(song.year.unwrap_or_default())
            .bind(song.content_type.clone().unwrap_or_default())
            .bind(song.file_size.unwrap_or_default())
            .bind(song.lyrics.clone().unwrap_or_default())
            .bind(song.updated_at)
            .bind(path_to_string(path));
            
            let result = query.execute(&self.pool).await?;
            if result.rows_affected() == 0 {
                tracing::info!("歌曲无更新结果 [{:?}] file_path: {}", result, path_to_string(path));
            }

            // 更新歌曲后也需要更新专辑统计信息（可能时长变化了）
            self.update_album_stats(album_id).await?;
            return Ok(());
        }

        sqlx::query(
            "INSERT INTO songs (id, album_id, artist_id, title, track_number, disc_number,
             duration, bit_rate, genre, year, content_type, file_path, file_size, lyrics, play_count,
             created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?)",
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
        .bind(&song.lyrics)
        .bind(song.created_at)
        .bind(song.updated_at)
        .execute(&self.pool)
        .await?;

        // 更新专辑的统计信息
        self.update_album_stats(album_id).await?;

        Ok(())
    }

    /// 更新专辑的统计信息（歌曲数量和总时长）
    async fn update_album_stats(&self, album_id: &str) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE albums
             SET song_count = (SELECT COUNT(*) FROM songs WHERE album_id = ?),
                 duration = (SELECT COALESCE(SUM(duration), 0) FROM songs WHERE album_id = ?),
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(album_id)
        .bind(album_id)
        .bind(album_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 创建新专辑
    async fn create_new_album(
        &self,
        artist_id: &str,
        name: &str,
        year: Option<i32>,
        genre: Option<&str>,
        cover_art_raw: Option<(String, Box<[u8]>)>,
    ) -> Result<String, AppError> {
        let mut album = Album::new(
            artist_id.to_string(),
            name.to_string(),
            path_to_string(&self.library_path),
            year,
            genre.map(|s| s.to_string()),
            None,
        );

        let mut cover_art_hash: Option<String> = None;

        // 处理封面图片
        if let Some((mime_type, data)) = cover_art_raw {
            // 计算 hash
            cover_art_hash = Some(calculate_image_hash(&data));

            // 处理图片
            let cover_art_id = self.process_cover_art(&mime_type, &data).await?;
            album.cover_art_path = Some(cover_art_id);
        }

        // 插入数据库（包含 hash）
        sqlx::query(
            "INSERT INTO albums (id, artist_id, name, year, genre, cover_art_path, cover_art_hash, path,
             song_count, duration, play_count, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0, ?, ?)",
        )
        .bind(&album.id)
        .bind(&album.artist_id)
        .bind(&album.name)
        .bind(&album.year)
        .bind(&album.genre)
        .bind(&album.cover_art_path)
        .bind(&cover_art_hash)
        .bind(&album.path)
        .bind(album.created_at)
        .bind(album.updated_at)
        .execute(&self.pool)
        .await?;

        tracing::debug!(
            "创建新专辑 [album_id={}, name={}, has_cover={}]",
            album.id,
            album.name,
            album.cover_art_path.is_some()
        );

        Ok(album.id)
    }

    /// 智能更新专辑信息（采用"已有值优先，只填充空值"策略）
    ///
    /// 策略说明：
    /// - year/genre: 如果数据库已有值，保持不变；只在为 NULL 时用新值填充
    /// - cover: 只在数据库完全没有封面时才写入新封面
    ///
    /// 这样可以避免因标签不一致导致的反复更新问题
    #[allow(clippy::too_many_arguments)]
    async fn update_album_if_needed(
        &self,
        album_id: &str,
        new_year: Option<i32>,
        new_genre: Option<&str>,
        existing_year: Option<i32>,
        existing_genre: Option<&str>,
        existing_cover_path: Option<String>,
        existing_cover_hash: Option<String>,
        cover_art_raw: Option<(String, Box<[u8]>)>,
    ) -> Result<(), AppError> {
        // 1. 采用"已有值优先"策略：只在数据库为空时才用新值填充
        let year_to_use = existing_year.or(new_year);
        let genre_to_use = existing_genre.or(new_genre);

        // 2. 检查是否有变化（从无到有才算变化）
        let year_changed = year_to_use != existing_year;
        let genre_changed = genre_to_use != existing_genre;

        // 3. 封面处理：只在数据库完全没有封面时才写入
        let mut new_cover_path = existing_cover_path.clone();
        let mut new_cover_hash = existing_cover_hash.clone();
        let mut cover_changed = false;

        // 只有在数据库没有任何封面信息时，才处理新封面
        if existing_cover_path.is_none() && existing_cover_hash.is_none() {
            if let Some((mime_type, data)) = cover_art_raw {
                tracing::info!(
                    "专辑封面从无到有 [album_id={}]",
                    album_id
                );

                let cover_art_id = self.process_cover_art(&mime_type, &data).await?;
                new_cover_path = Some(cover_art_id.clone());
                new_cover_hash = Some(calculate_image_hash(&data));
                cover_changed = true;
            }
        } else {
            // 已有封面，不再更新
            tracing::debug!(
                "专辑已有封面，保持不变 [album_id={}]",
                album_id
            );
        }

        // 4. 只在有任何变化时才执行 UPDATE
        if year_changed || genre_changed || cover_changed {
            let changes: Vec<&str> = [
                year_changed.then_some("year"),
                genre_changed.then_some("genre"),
                cover_changed.then_some("cover"),
            ]
            .iter()
            .filter_map(|&x| x)
            .collect();

            tracing::info!(
                "填充专辑空值 [album_id={}]: 变更字段={:?}",
                album_id,
                changes
            );

            sqlx::query(
                "UPDATE albums
                 SET year = ?, genre = ?, cover_art_path = ?, cover_art_hash = ?, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?",
            )
            .bind(year_to_use)
            .bind(genre_to_use)
            .bind(&new_cover_path)
            .bind(&new_cover_hash)
            .bind(album_id)
            .execute(&self.pool)
            .await?;
        } else {
            tracing::debug!("专辑信息无需填充 [album_id={}]", album_id);
        }

        Ok(())
    }

    /// 处理封面图片：保存原图 + 预热缓存
    async fn process_cover_art(&self, mime_type: &str, data: &[u8]) -> Result<String, AppError> {
        let format = get_image_format(mime_type);
        let cover_art_id = format!("al-{}", Uuid::new_v4().to_string()[0..8].to_string());

        // 创建 originals 目录
        let original_dir = PathBuf::from("./coverArt/originals");
        if !original_dir.exists() {
            std::fs::create_dir_all(&original_dir)?;
        }

        // 保存原始图片
        let original_file_path = format!("./coverArt/originals/{}.{}", cover_art_id, format);
        write_image_to_file(data, &original_file_path)?;

        // 预生成 300px WebP 缓存
        if let Err(e) = self
            .prewarm_cover_cache(&cover_art_id, &original_file_path)
            .await
        {
            tracing::warn!("预热封面缓存失败 [cover_id={}]: {}", cover_art_id, e);
        }

        Ok(cover_art_id)
    }

    /// 预热封面缓存
    async fn prewarm_cover_cache(
        &self,
        cover_art_id: &str,
        original_path: &str,
    ) -> Result<(), std::io::Error> {
        const DEFAULT_SIZE: u32 = 300;
        const CACHE_PATH: &str = "./coverArt/webp";

        let webp_dir = PathBuf::from(CACHE_PATH);
        if !webp_dir.exists() {
            std::fs::create_dir_all(&webp_dir)?;
        }

        let cache_path = crate::utils::image_utils::get_webp_cache_path(cover_art_id, DEFAULT_SIZE);

        let original_path = original_path.to_string();
        let cache_path_clone = cache_path.clone();

        tokio::task::spawn_blocking(move || {
            let config = crate::utils::image_utils::WebPConfig::default();
            crate::utils::image_utils::resize_and_convert_to_webp(
                Path::new(&original_path),
                &cache_path_clone,
                DEFAULT_SIZE,
                &config,
            )
        })
        .await
        .map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Task join error: {}", e))
        })??;

        tracing::info!("Prewarmed cover cache: {} (300px)", cover_art_id);
        Ok(())
    }

    /// 清理数据库中文件已不存在的歌曲
    async fn cleanup_deleted_files(&self) -> Result<usize, AppError> {
        // 获取所有歌曲的文件路径
        let all_songs = sqlx::query_as::<_, (String, String)>(
            "SELECT id, file_path FROM songs"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut deleted_count = 0;
        let mut deleted_song_ids = Vec::new();

        for (song_id, file_path) in all_songs {
            let path = Path::new(&file_path);
            if !path.exists() {
                tracing::debug!("发现已删除的文件: {}", file_path);
                deleted_song_ids.push(song_id);
                deleted_count += 1;
            }
        }

        // 批量删除不存在的歌曲
        if !deleted_song_ids.is_empty() {
            for song_id in &deleted_song_ids {
                sqlx::query("DELETE FROM songs WHERE id = ?")
                    .bind(song_id)
                    .execute(&self.pool)
                    .await?;
            }

            tracing::info!("清理了 {} 个已删除的歌曲文件", deleted_count);

            // 清理空专辑和空艺术家
            self.cleanup_empty_albums().await?;
            self.cleanup_empty_artists().await?;
        }

        Ok(deleted_count)
    }

    /// 清理没有歌曲的专辑
    async fn cleanup_empty_albums(&self) -> Result<usize, AppError> {
        let result = sqlx::query(
            "DELETE FROM albums
             WHERE id NOT IN (SELECT DISTINCT album_id FROM songs)"
        )
        .execute(&self.pool)
        .await?;

        let deleted_count = result.rows_affected() as usize;
        if deleted_count > 0 {
            tracing::info!("清理了 {} 个空专辑", deleted_count);
        }

        Ok(deleted_count)
    }

    /// 清理没有专辑的艺术家
    async fn cleanup_empty_artists(&self) -> Result<usize, AppError> {
        let result = sqlx::query(
            "DELETE FROM artists
             WHERE id NOT IN (SELECT DISTINCT artist_id FROM albums)"
        )
        .execute(&self.pool)
        .await?;

        let deleted_count = result.rows_affected() as usize;
        if deleted_count > 0 {
            tracing::info!("清理了 {} 个空艺术家", deleted_count);
        }

        Ok(deleted_count)
    }
}

/// 计算图片数据的 SHA256 hash，性能开销不大，耗时在毫秒级
fn calculate_image_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// 手动测试入口:可以通过环境变量指定音频文件路径
    ///
    /// # 使用示例
    /// ```
    /// TEST_AUDIO_FILE=/path/to/your/audio.mp3 cargo test test_read_custom_audio -- --nocapture
    /// ```
    #[tokio::test]
    async fn test_read_custom_audio() {
        let test_file = std::env::var("TEST_AUDIO_FILE")
            .unwrap_or_else(|_| "./music/Evanescence - Bring Me To Life.mp3".to_string());

        if test_file.is_empty() {
            return;
        }

        let test_path = PathBuf::from(&test_file);
        if !test_path.exists() {
            println!("测试文件不存在: {:?}", test_path);
            return;
        }

        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("创建测试数据库失败");

        let scan_service = ScanService::new(pool, PathBuf::from("./"));

        match scan_service.read_audio_metadata(&test_path).await {
            Ok(metadata) => {
                println!("\n========== 音频文件元数据 ==========");
                println!("文件路径: {}", test_file);
                println!("标题: {:?}", metadata.title);
                println!("艺术家: {:?}", metadata.artist);
                println!("专辑: {:?}", metadata.album);
                println!("专辑艺术家: {:?}", metadata.album_artist);
                println!("流派: {:?}", metadata.genre);
                println!("年份: {:?}", metadata.year);
                println!("音轨号: {:?}", metadata.track_number);
                println!("光盘号: {:?}", metadata.disc_number);
                println!("时长: {} 秒", metadata.duration_secs);
                println!("比特率: {:?} bps", metadata.bit_rate);
                println!("采样率: {:?} Hz", metadata.sample_rate);
                println!("声道数: {:?}", metadata.channels);
                println!("内容类型: {}", metadata.content_type);
                println!("文件大小: {:?} bytes", metadata.file_size);

                if let Some(lyrics) = metadata.lyrics {
                    println!("歌词大小: {}", lyrics.len())
                }

                if let Some((mime, data)) = metadata.cover_art_raw {
                    println!("封面: {} ({} bytes)", mime, data.len());
                }

                println!("=====================================\n");
            }
            Err(e) => {
                eprintln!("读取音频元数据失败: {}", e);
                panic!("测试失败");
            }
        }
    }
}
