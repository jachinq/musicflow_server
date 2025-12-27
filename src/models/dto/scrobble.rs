//! 播放记录数据传输对象
#![allow(dead_code)]

use serde::Deserialize;

/// 播放记录请求
#[derive(Debug, Deserialize)]
pub struct ScrobbleRequest {
    pub id: String,
    pub submission: Option<bool>,
    pub time: Option<i64>,
}
