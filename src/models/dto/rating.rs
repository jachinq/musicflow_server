//! 评分数据传输对象
#![allow(dead_code)]

use serde::Deserialize;

/// 设置评分请求
#[derive(Debug, Deserialize)]
pub struct SetRatingRequest {
    pub id: String,
    pub rating: i32,
}

/// 获取评分请求
#[derive(Debug, Deserialize)]
pub struct GetRatingRequest {
    pub id: String,
}
