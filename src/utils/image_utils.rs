#![allow(dead_code)]

use crate::error::AppError;
use anyhow::Result;
use axum::body::Body;
use axum::{extract::Query, http::HeaderMap, response::IntoResponse, routing::get, Router};
use image::DynamicImage;
use image::{imageops::FilterType, GenericImageView, ImageFormat};
use once_cell::sync::Lazy;
use sqlx::SqlitePool;
use tokio_util::io::ReaderStream;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;
use tokio::sync::Mutex;

/// 音频元数据
#[derive(Debug, Default)]
pub struct AudioMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub duration_secs: u64,
    pub bit_rate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub channels: Option<u8>,
    pub content_type: String,
    pub file_size: Option<u64>,
    pub cover_art_raw: Option<(String, Box<[u8]>)>,
    pub lyrics: Option<String>,
}

/// write image to file
pub fn write_image_to_file(image: &[u8], file_path: &str) -> Result<(), std::io::Error> {
    let mut file = std::fs::File::create(file_path)?;
    file.write_all(image)?;
    Ok(())
}

/// get image format
pub fn get_image_format(mime: &str) -> &str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        _ => "jpg",
    }
}

/// WebP 配置
pub struct WebPConfig {
    pub quality: f32, // 0.0 - 100.0
}

impl Default for WebPConfig {
    fn default() -> Self {
        Self { quality: 50.0 }
    }
}

/// 缩放并转换为 WebP
///
/// # 参数
/// - `input_path`: 原图路径
/// - `output_path`: 输出 WebP 路径
/// - `target_size`: 目标尺寸（宽高中的最大值）
/// - `config`: WebP 配置
pub fn resize_and_convert_to_webp(
    input_path: &Path,
    output_path: &Path,
    target_size: u32,
    config: &WebPConfig,
) -> Result<(), std::io::Error> {
    // 读取原图
    let img = image::open(input_path).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to open image: {} input={}", e, input_path.display()),
        )
    })?;

    // 计算缩放尺寸（保持宽高比，不放大）
    let (width, height) = img.dimensions();
    let (new_width, new_height) = calculate_resize_dimensions(width, height, target_size);

    // 缩放（Lanczos3 高质量滤波器）
    let resized = img.resize_exact(new_width, new_height, FilterType::Lanczos3);

    // 转换为 WebP
    let webp = compress_img(&resized, config.quality);
    if webp.is_err() {
        // 压缩失败，保存为不压缩的 WebP
        resized
            .save_with_format(output_path, ImageFormat::WebP)
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to save WebP: {}", e),
                )
            })?;
    } else {
        let output_path = output_path.to_string_lossy().to_string();
        write_image_to_file(&webp.unwrap(), &output_path).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to compress and save WebP: {}", e),
            )
        })?;
    }

    Ok(())
}

pub fn resize_and_convert_to_webp_by_data(
    data: &[u8],
    output_path: &Path,
    target_size: u32,
    config: &WebPConfig,
) -> Result<(), std::io::Error> {
    // 读取原图
    let img = image::load_from_memory(data).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Failed to open image: {} output={}",
                e,
                output_path.display()
            ),
        )
    })?;

    // 计算缩放尺寸（保持宽高比，不放大）
    let (width, height) = img.dimensions();
    let (new_width, new_height) = calculate_resize_dimensions(width, height, target_size);

    // 缩放（Lanczos3 高质量滤波器）
    let resized = img.resize_exact(new_width, new_height, FilterType::Lanczos3);

    // 转换为 WebP
    let webp = compress_img(&resized, config.quality);
    if webp.is_err() {
        // 压缩失败，保存为不压缩的 WebP
        resized
            .save_with_format(output_path, ImageFormat::WebP)
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to save WebP: {}", e),
                )
            })?;
    } else {
        let output_path = output_path.to_string_lossy().to_string();
        write_image_to_file(&webp.unwrap(), &output_path).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to compress and save WebP: {}", e),
            )
        })?;
    }

    Ok(())
}

/// 根据 img 压缩成 webp 格式
pub fn compress_img(img: &DynamicImage, qulity: f32) -> Result<Vec<u8>, String> {
    match webp::Encoder::from_image(img) {
        Err(err) => Err(err.to_string()),
        Ok(encoder) => {
            let webp = encoder.encode(qulity);
            Ok(webp.to_vec())
        }
    }
}

/// 计算缩放尺寸（保持宽高比，不放大）
fn calculate_resize_dimensions(
    original_width: u32,
    original_height: u32,
    target_size: u32,
) -> (u32, u32) {
    let max_dimension = original_width.max(original_height);

    // 如果原图已经小于等于目标尺寸，不放大
    if max_dimension <= target_size {
        return (original_width, original_height);
    }

    // 保持宽高比缩放
    let scale = target_size as f32 / max_dimension as f32;
    (
        (original_width as f32 * scale).round() as u32,
        (original_height as f32 * scale).round() as u32,
    )
}

/// 获取 WebP 缓存路径
pub fn get_webp_cache_path(cover_art_id: &str, size: u32) -> PathBuf {
    PathBuf::from(format!("./coverArt/webp/{}_{}.webp", cover_art_id, size))
}

/// 获取原图路径（兼容新旧格式）
pub fn get_original_image_path(cover_art_id: &str) -> Option<PathBuf> {
    // 尝试新格式: ./coverArt/originals/{id}.{ext}
    for ext in &["jpg", "jpeg", "png", "gif"] {
        let path = PathBuf::from(format!("./coverArt/originals/{}.{}", cover_art_id, ext));
        if path.exists() {
            return Some(path);
        }
    }

    // 兼容旧格式: ./coverArt/{id}.{ext}
    for ext in &["jpg", "jpeg", "png", "gif"] {
        let path = PathBuf::from(format!("./coverArt/{}.{}", cover_art_id, ext));
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// 提取音频元数据
pub fn extract_audio_metadata_static(path: &Path) -> Result<AudioMetadata, AppError> {
    // 打开文件
    let file =
        File::open(path).map_err(|e| AppError::ValidationError(format!("无法打开文件: {}", e)))?;

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
                        if let Ok(year) = tag.value.to_string().parse::<i32>() {
                            metadata.year = Some(year);
                        } else {
                            if let Some(year_str) = tag.value.to_string().split('-').next() {
                                if let Ok(year) = year_str.parse::<i32>() {
                                    metadata.year = Some(year);
                                }
                            }
                        }
                    }
                    Some(StandardTagKey::TrackNumber) => {
                        let track_str = tag.value.to_string();
                        if let Some(track_num) = track_str.split('/').next() {
                            if let Ok(num) = track_num.parse::<i32>() {
                                metadata.track_number = Some(num);
                            }
                        }
                    }
                    Some(StandardTagKey::DiscNumber) => {
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

            album.iter().for_each(|f| {
                metadata.cover_art_raw = Some((f.0.to_string(), f.1.clone()));
            });
        }
    }

    Ok(metadata)
}

/// 静态方法:预热封面缓存
pub async fn prewarm_cover_cache_static(
    cover_art_id: &str,
    // original_path: &str,
    original_data: Box<[u8]>,
) -> Result<(), std::io::Error> {
    const DEFAULT_SIZE: u32 = 300;
    const CACHE_PATH: &str = "./coverArt/webp";

    let webp_dir = PathBuf::from(CACHE_PATH);
    if !webp_dir.exists() {
        std::fs::create_dir_all(&webp_dir)?;
    }

    let cache_path = crate::utils::image_utils::get_webp_cache_path(cover_art_id, DEFAULT_SIZE);
    // let original_path = original_path.to_string();
    let cache_path_clone = cache_path.clone();

    tokio::task::spawn_blocking(move || {
        let config = crate::utils::image_utils::WebPConfig::default();
        crate::utils::image_utils::resize_and_convert_to_webp_by_data(
            &original_data,
            &cache_path_clone,
            DEFAULT_SIZE,
            &config,
        )
    })
    .await
    .map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Task join error: {}", e))
    })??;

    Ok(())
}

// 防止同一封面同一尺寸被多次生成
// Key: "{cover_art_id}_{size}"
static CACHE_GENERATION_LOCKS: Lazy<Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
/// 异步方法:预热封面缓存
pub async fn prewarm_cover_from_original(
    cover_art_id: &str,
    size: u32,
    original_path: &Path,
) -> Result<()> {
    // 3. 获取生成锁
    let lock_key = format!("{}_{}", cover_art_id, size);
    let generation_lock = {
        let mut locks = CACHE_GENERATION_LOCKS.lock().await;
        locks
            .entry(lock_key.clone())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    };

    let _guard = generation_lock.lock().await;

    // 4. 双重检查缓存
    let cache_path = get_webp_cache_path(cover_art_id, size);
    if cache_path.exists() {
        return Ok(());
    }

    // 5. 创建缓存目录
    let webp_dir = PathBuf::from("./coverArt/webp");
    if !webp_dir.exists() {
        std::fs::create_dir_all(&webp_dir).map_err(|e| AppError::IoError(e))?;
    }

    // 6. 生成 WebP 缓存（spawn_blocking 避免阻塞）
    let mut config = WebPConfig::default();
    if size > 300 {
        config.quality = 75.0; // 默认 50，超过默认尺寸的图片用 75 质量
    }
    tracing::info!(
        "Generating WebP cache: {} -> {}",
        original_path.display(),
        cache_path.display()
    );
    resize_and_convert_to_webp(&original_path, &cache_path, size, &config).map_err(|e| {
        AppError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Task join error: {}", e),
        ))
    })?;
    Ok(())
}

/// 服务图片文件
pub async fn serve_image_file(file_path: PathBuf) -> Result<impl IntoResponse, AppError> {
    let file = tokio::fs::File::open(&file_path)
        .await
        .map_err(|e| AppError::IoError(e))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // 根据文件扩展名确定 Content-Type
    let ext = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let content_type = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    };

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", content_type.parse().unwrap());

    Ok((headers, body).into_response())
}
