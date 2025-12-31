//! 音乐库扫描服务
// #![allow(dead_code)]

use crate::error::AppError;
use crate::handlers::library::ScanState;
use crate::models::entities::{Album, Artist, Song};
use crate::utils::{get_image_format, image_utils, write_image_to_file, AudioMetadata};
use sha2::{Digest, Sha256};
use sqlx::{Execute, SqlitePool};
use std::fs::File;
use std::path::{Path, PathBuf};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;
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

impl ScanService {
    pub fn new(pool: SqlitePool, library_path: PathBuf) -> Self {
        Self { pool, library_path }
    }

    /// 扫描音乐库 (优化版: 并发处理 + 批量插入 + 增量扫描)
    pub async fn scan_library(&self, scan_state: ScanState) -> Result<ScanResult, AppError> {
        use futures::stream::{self, StreamExt};

        let mut result = ScanResult::default();

        if !self.library_path.exists() {
            return Err(AppError::not_found("Music library path"));
        }

        let scan_start = std::time::Instant::now();
        tracing::info!("开始扫描音乐库: {:?}", self.library_path);

        // 步骤1: 收集所有音频文件路径
        let mut paths = vec![];
        for entry in WalkDir::new(&self.library_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let ext = entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            // 支持的音频格式
            if matches!(
                ext.as_str(),
                "mp3" | "flac" | "wav" | "m4a" | "aac" | "ogg" | "opus"
            ) {
                paths.push(entry.into_path());
            }
        }

        let total_files = paths.len();

        tracing::info!("发现 {} 个音频文件", total_files);

        // 步骤1.5: 增量扫描优化 - 查询数据库中已存在的文件及其更新时间
        let db_files = self.get_existing_files_info().await?;
        tracing::info!("数据库中已有 {} 个文件记录", db_files.len());

        // 过滤需要扫描的文件(新增或修改的文件)
        let mut files_to_scan = Vec::new();
        let mut skipped = 0;

        for path in paths {
            let path_str = path_to_string(&path);

            // 获取文件修改时间
            let file_mtime = match std::fs::metadata(&path) {
                Ok(metadata) => metadata.modified().ok(),
                Err(_) => None,
            };

            // 检查是否需要扫描
            let should_scan = match db_files.get(&path_str) {
                Some(db_updated_at) => {
                    // 文件存在于数据库,比较修改时间
                    if let Some(mtime) = file_mtime {
                        // 将数据库时间字符串转换为系统时间
                        match chrono::DateTime::parse_from_rfc3339(db_updated_at) {
                            Ok(db_time) => {
                                let file_time = chrono::DateTime::<chrono::Utc>::from(mtime);
                                // 如果文件修改时间晚于数据库更新时间,需要重新扫描
                                file_time > db_time
                            }
                            Err(_) => true, // 解析失败,重新扫描
                        }
                    } else {
                        true // 无法获取文件时间,重新扫描
                    }
                }
                None => true, // 新文件,需要扫描
            };

            if should_scan {
                files_to_scan.push(path);
            } else {
                skipped += 1;
            }
        }

        tracing::info!(
            "增量扫描: 需处理 {} 个文件, 跳过 {} 个未修改文件 (节省 {:.1}%)",
            files_to_scan.len(),
            skipped,
            (skipped as f64 / total_files as f64) * 100.0
        );

        // 更新总数为实际需要处理的文件数
        let mut count = scan_state.count.lock().await;
        *count = files_to_scan.len();
        drop(count);

        if files_to_scan.is_empty() {
            tracing::info!("所有文件都是最新的,无需扫描");

            // 仍然需要清理已删除的文件
            let deleted = self.cleanup_deleted_files().await?;
            result.deleted = deleted;

            // 更新统计
            let artist_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM artists")
                .fetch_one(&self.pool)
                .await? as usize;
            let album_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM albums")
                .fetch_one(&self.pool)
                .await? as usize;
            let song_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM songs")
                .fetch_one(&self.pool)
                .await? as usize;

            result.artists = artist_count;
            result.albums = album_count;
            result.songs = song_count;

            return Ok(result);
        }

        tracing::info!("开始并发处理 {} 个文件", files_to_scan.len());

        // 步骤2: 并发解析元数据 (CPU密集型任务)
        const CONCURRENT_PARSE: usize = 8; // 并发解析数量
        const BATCH_SIZE: usize = 100; // 批量插入大小

        let mut metadata_stream = stream::iter(files_to_scan.into_iter().enumerate())
            .map(|(index, path)| {
                async move {
                    let path_clone = path.clone();
                    // 在阻塞线程池中解析元数据(CPU密集型)
                    let result = tokio::task::spawn_blocking(move || {
                        image_utils::extract_audio_metadata_static(&path_clone)
                    })
                    .await;

                    match result {
                        Ok(Ok(metadata)) => Ok((index, path, metadata)),
                        Ok(Err(e)) => Err((index, path.clone(), e)),
                        Err(e) => Err((
                            index,
                            path.clone(),
                            AppError::ValidationError(format!("解析任务失败: {}", e)),
                        )),
                    }
                }
            })
            .buffer_unordered(CONCURRENT_PARSE);

        // 步骤3: 批量收集并插入数据库
        let mut batch = Vec::with_capacity(BATCH_SIZE);
        let mut processed = 0;

        while let Some(parse_result) = metadata_stream.next().await {
            processed += 1;

            match parse_result {
                Ok((_index, path, metadata)) => {
                    batch.push((path, metadata));

                    // 批量插入
                    if batch.len() >= BATCH_SIZE {
                        let batch_result = self.batch_save_to_database(&batch).await;
                        match batch_result {
                            Ok(count) => result.songs += count,
                            Err(e) => {
                                tracing::error!("批量插入失败: {}", e);
                                result.failed += batch.len();
                            }
                        }
                        batch.clear();
                    }
                }
                Err((index, path, e)) => {
                    tracing::warn!(
                        "[{}/{}] 解析失败 {}: {}",
                        index + 1,
                        total_files,
                        path.display(),
                        e
                    );
                    result.failed += 1;
                }
            }

            // 更新进度 (每10个文件更新一次,减少锁竞争)
            if processed % 10 == 0 || processed == total_files {
                let mut current = scan_state.current.lock().await;
                *current = processed;
                drop(current); // unlock

                // 每100个文件输出一次进度日志
                if processed % 100 == 0 || processed == total_files {
                    let elapsed = scan_start.elapsed().as_secs_f64();
                    let speed = processed as f64 / elapsed;
                    tracing::info!(
                        "进度: {}/{} ({:.1}%) - 速度: {:.1} 文件/秒",
                        processed,
                        total_files,
                        (processed as f64 / total_files as f64) * 100.0,
                        speed
                    );
                }
            }
        }

        // 步骤4: 处理剩余批次
        if !batch.is_empty() {
            match self.batch_save_to_database(&batch).await {
                Ok(count) => result.songs += count,
                Err(e) => {
                    tracing::error!("最后批次插入失败: {}", e);
                    result.failed += batch.len();
                }
            }
        }

        // 步骤5: 清理已删除的文件
        let deleted = self.cleanup_deleted_files().await?;
        result.deleted = deleted;

        // 步骤6: 更新艺术家和专辑计数
        let artist_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM artists")
            .fetch_one(&self.pool)
            .await? as usize;
        let album_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM albums")
            .fetch_one(&self.pool)
            .await? as usize;

        result.artists = artist_count;
        result.albums = album_count;

        let total_time = scan_start.elapsed();
        let avg_speed = total_files as f64 / total_time.as_secs_f64();

        tracing::info!(
            "扫描完成: {:?} | 总耗时: {:.2}s | 平均速度: {:.1} 文件/秒",
            result,
            total_time.as_secs_f64(),
            avg_speed
        );

        Ok(result)
    }

    /// 获取数据库中已存在文件的路径和更新时间映射
    async fn get_existing_files_info(
        &self,
    ) -> Result<std::collections::HashMap<String, String>, AppError> {
        use std::collections::HashMap;

        let rows = sqlx::query_as::<_, (String, String)>("SELECT file_path, updated_at FROM songs")
            .fetch_all(&self.pool)
            .await?;

        let mut map = HashMap::with_capacity(rows.len());
        for (path, updated_at) in rows {
            map.insert(path, updated_at);
        }

        Ok(map)
    }

    /// 批量保存到数据库 (优化版: 减少数据库往返 + 封面异步处理)
    async fn batch_save_to_database(
        &self,
        batch: &[(PathBuf, AudioMetadata)],
    ) -> Result<usize, AppError> {
        if batch.is_empty() {
            return Ok(0);
        }

        // 收集需要处理的封面(延迟到事务外处理)
        let mut pending_covers: Vec<(String, String, Box<[u8]>)> = Vec::new();

        // 使用事务批量处理
        let mut tx = self.pool.begin().await?;
        let mut success_count = 0;

        for (path, metadata) in batch {
            let artist_name_fallback = self.extract_artist_from_path(path);
            let album_name_fallback = self.extract_album_from_path(path);

            let artist_name = metadata.artist.as_deref().unwrap_or(&artist_name_fallback);
            let album_name = metadata.album.as_deref().unwrap_or(&album_name_fallback);
            let title = metadata.title.as_deref().unwrap_or("Unknown");

            // 使用事务内的连接进行插入(不处理封面)
            let result = self
                .save_to_database_tx_deferred_cover(
                    &mut tx,
                    &mut pending_covers,
                    artist_name,
                    album_name,
                    title,
                    path,
                    metadata.duration_secs as i32,
                    metadata.bit_rate,
                    metadata.year,
                    metadata.genre.as_deref(),
                    metadata.track_number,
                    metadata.disc_number,
                    &metadata.content_type,
                    metadata.file_size,
                    metadata.cover_art_raw.clone(),
                    metadata.lyrics.as_deref(),
                )
                .await;

            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    tracing::warn!("保存失败 {}: {}", path.display(), e);
                }
            }
        }

        // 提交事务
        tx.commit().await?;

        // 事务提交后,并发处理封面
        if !pending_covers.is_empty() {
            tracing::info!("开始并发处理 {} 个封面", pending_covers.len());
            let start = std::time::Instant::now();
            self.process_covers_concurrently(pending_covers).await;
            tracing::info!(
                "当前封面并发处理完成,耗时: {:.2}s",
                start.elapsed().as_secs_f64()
            );
        }

        Ok(success_count)
    }

    /// 并发处理多个封面
    async fn process_covers_concurrently(&self, covers: Vec<(String, String, Box<[u8]>)>) {
        // 1. 并发写图片到文件系统
        use futures::stream::{self, StreamExt};
        const CONCURRENT_COVERS: usize = 4;
        let results: Vec<_> = stream::iter(covers)
            .map(|(album_id, mime_type, data)| async move {
                match Self::process_cover_art_for_album(&album_id, &mime_type, data).await {
                    Ok(cover_art) => Some((album_id, cover_art)),
                    Err(e) => {
                        tracing::warn!("处理封面失败 [album_id={}]: {}", album_id, e);
                        None
                    }
                }
            })
            .buffer_unordered(CONCURRENT_COVERS)
            .collect()
            .await;

        // 2. 统一更新数据库中的封面路径，单线程，因为 sqlite 对并发支持不好
        for (album_id, cover_art_id) in results.into_iter().flatten() {
            let _ = sqlx::query("UPDATE albums SET cover_art_path = ? WHERE id = ?")
                .bind(&cover_art_id)
                .bind(album_id)
                .execute(&self.pool)
                .await;
        }
    }

    /// 为指定专辑处理封面
    /// 1. 同步写入原始图片，主要耗时
    /// 2. 异步预热缓存(不等待完成)
    async fn process_cover_art_for_album(
        album_id: &str,
        mime_type: &str,
        original_data: Box<[u8]>,
    ) -> Result<String, AppError> {
        let cover_art_id = format!("al-{}", &album_id[0..8]);
        let format = get_image_format(mime_type).to_string(); // 转换为拥有的String

        let start = std::time::Instant::now();
        // 创建 originals 目录
        let original_dir = PathBuf::from("./coverArt/originals");
        if !original_dir.exists() {
            std::fs::create_dir_all(&original_dir)?;
        }

        // 保存原始图片
        let original_file_path = format!("./coverArt/originals/{}.{}", cover_art_id, format);
        write_image_to_file(&original_data, &original_file_path)?;

        tracing::debug!("写入原图成功，耗时{:.2}", start.elapsed().as_secs_f64());
        let start = std::time::Instant::now();

        // 异步预热缓存(不等待完成)
        let cover_art_id_clone = cover_art_id.clone();
        tokio::spawn(async move {
            if let Err(e) =
                image_utils::prewarm_cover_cache_static(&cover_art_id_clone, original_data).await
            {
                tracing::warn!("预热封面缓存失败 [cover_id={}]: {}", cover_art_id_clone, e);
            }
        });
        tracing::debug!("异步缓存图片成功，耗时{:.2}", start.elapsed().as_secs_f64());

        Ok(cover_art_id)
    }

    /// 在事务中保存到数据库(封面延迟处理版本)
    #[allow(clippy::too_many_arguments)]
    async fn save_to_database_tx_deferred_cover(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        pending_covers: &mut Vec<(String, String, Box<[u8]>)>,
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
        let artist_id = self.get_or_create_artist_tx(tx, artist_name).await?;

        // 插入或更新专辑(不处理封面)
        let album_id = self
            .get_or_create_album_tx_no_cover(tx, &artist_id, album_name, year, genre)
            .await?;

        // 如果有封面数据,添加到待处理列表
        if let Some((mime_type, data)) = cover_art_raw {
            pending_covers.push((album_id.clone(), mime_type, data));
        }

        // 插入或更新歌曲
        self.get_or_create_song_tx(
            tx,
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

    /// 获取或创建专辑(事务版本,不处理封面)
    async fn get_or_create_album_tx_no_cover(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        artist_id: &str,
        name: &str,
        year: Option<i32>,
        genre: Option<&str>,
    ) -> Result<String, AppError> {
        // 查找已存在的专辑
        let existing = sqlx::query_as::<_, (String, Option<i32>, Option<String>)>(
            "SELECT id, year, genre FROM albums WHERE artist_id = ? AND name = ?",
        )
        .bind(artist_id)
        .bind(name)
        .fetch_optional(&mut **tx)
        .await?;

        if let Some((album_id, existing_year, existing_genre)) = existing {
            // 更新元数据(不包括封面)
            let year_to_use = existing_year.or(year);
            let genre_to_use = existing_genre.as_deref().or(genre);

            if year_to_use != existing_year || genre_to_use != existing_genre.as_deref() {
                sqlx::query(
                    "UPDATE albums SET year = ?, genre = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                )
                .bind(year_to_use)
                .bind(genre_to_use)
                .bind(&album_id)
                .execute(&mut **tx)
                .await?;
            }

            return Ok(album_id);
        }

        // 创建新专辑(不包含封面)
        let album = Album::new(
            artist_id.to_string(),
            name.to_string(),
            path_to_string(&self.library_path),
            year,
            genre.map(|s| s.to_string()),
            None,
        );

        sqlx::query(
            "INSERT INTO albums (id, artist_id, name, year, genre, cover_art_path, cover_art_hash, path,
             song_count, duration, play_count, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, NULL, NULL, ?, 0, 0, 0, ?, ?)",
        )
        .bind(&album.id)
        .bind(&album.artist_id)
        .bind(&album.name)
        .bind(album.year)
        .bind(&album.genre)
        .bind(&album.path)
        .bind(album.created_at)
        .bind(album.updated_at)
        .execute(&mut **tx)
        .await?;

        Ok(album.id)
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

    /// 获取或创建艺术家 (事务版本)
    async fn get_or_create_artist_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        name: &str,
    ) -> Result<String, AppError> {
        // 先尝试查找
        let existing = sqlx::query_scalar::<_, String>("SELECT id FROM artists WHERE name = ?")
            .bind(name)
            .fetch_optional(&mut **tx)
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
        .execute(&mut **tx)
        .await?;

        Ok(artist.id)
    }

    /// 获取或创建歌曲 (事务版本)
    #[allow(clippy::too_many_arguments)]
    async fn get_or_create_song_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
        let existing = sqlx::query_scalar::<_, String>("SELECT id FROM songs WHERE file_path = ?")
            .bind(path_to_string(path))
            .fetch_optional(&mut **tx)
            .await?;

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
            sqlx::query(
                "UPDATE songs
                 SET title = ?, track_number = ?, disc_number = ?, duration = ?, bit_rate = ?,
                     genre = ?, year = ?, content_type = ?, file_size = ?, lyrics = ?, updated_at = ?
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
            .bind(path_to_string(path))
            .execute(&mut **tx)
            .await?;

            self.update_album_stats_tx(tx, album_id).await?;
        } else {
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
            .bind(song.track_number)
            .bind(song.disc_number)
            .bind(song.duration)
            .bind(song.bit_rate)
            .bind(&song.genre)
            .bind(song.year)
            .bind(&song.content_type)
            .bind(&song.file_path)
            .bind(song.file_size)
            .bind(&song.lyrics)
            .bind(song.created_at)
            .bind(song.updated_at)
            .execute(&mut **tx)
            .await?;

            self.update_album_stats_tx(tx, album_id).await?;
        }

        Ok(())
    }

    /// 更新专辑统计 (事务版本)
    async fn update_album_stats_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        album_id: &str,
    ) -> Result<(), AppError> {
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
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 清理数据库中文件已不存在的歌曲
    async fn cleanup_deleted_files(&self) -> Result<usize, AppError> {
        // 获取所有歌曲的文件路径
        let all_songs = sqlx::query_as::<_, (String, String)>("SELECT id, file_path FROM songs")
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
             WHERE id NOT IN (SELECT DISTINCT album_id FROM songs)",
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
             WHERE id NOT IN (SELECT DISTINCT artist_id FROM albums)",
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

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        // 2. 初始化日志
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new("debug"))
            .with(
                tracing_subscriber::fmt::layer()
                    .with_file(true) // 显示文件名
                    .with_line_number(true) // 显示行号
                    .with_target(true), // 显示模块路径(target)
            )
            .init();
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

        match image_utils::extract_audio_metadata_static(&test_path) {
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

                    // let album_id = id_builder::generate_id();
                    // let result =
                    //     ScanService::process_cover_art_for_album(&album_id, &mime, data).await;

                    // // 等待 3s 异步图片处理任务结束
                    // tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    // println!("封面处理结果: album_id={} result={:?}", album_id, result);
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
