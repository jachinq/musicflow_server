#![allow(dead_code)]

use image::DynamicImage;
use image::{imageops::FilterType, GenericImageView, ImageFormat};
use std::io::Write;
use std::path::{Path, PathBuf};

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
    config: &WebPConfig,) -> Result<(), std::io::Error> {
    // 读取原图
    let img = image::load_from_memory(data).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to open image: {} output={}", e, output_path.display()),
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
