//! 用于生成随机 ID
//! 基于概率公式，数量级对应的推荐 id 长度如下
//! 10万数量 -> 10
//! 30万数量 -> 11
//! 百万数量 -> 12
//! 千万数量 -> 13-14
//! 几乎无限 -> 16+
#![allow(dead_code)]
use std::fmt::format;

use uuid::Uuid;

pub fn generate_id_by_len(len: u32) -> String {
    Uuid::new_v4()
        .to_string()
        .replace("-", "")
        .chars()
        .take(len as usize)
        .collect()
}

pub fn generate_id() -> String {
    generate_id_by_len(16) // 默认长度为 16，足够安全
}

pub enum CoverArt {
    Album,
    Artist,
}

impl CoverArt {
    pub fn get_id(&self, id: &str) -> String {
        let prefix = match self {
            CoverArt::Album => "al",
            CoverArt::Artist => "ar",
        };
        format!("{}-{}", prefix, id)
    }
}

pub fn gen_cover_id(c: CoverArt) -> String {
    c.get_id(&generate_id())
}

#[test]
fn test_generate_id() {
    for l in 8..=32 {
        let id = generate_id_by_len(l);
        println!("len: {} id: {}", l, id);
        assert_eq!(id.len(), l as usize);
    }

    let id = generate_id();
    println!("default id: {}", id);
    assert_eq!(id.len(), 16);

    let al_id = gen_cover_id(CoverArt::Album);
    println!("album id: {}", al_id);
    assert_eq!(al_id.len(), 19);

    let ar_id = gen_cover_id(CoverArt::Artist);
    println!("artist id: {}", ar_id);
    assert_eq!(ar_id.len(), 19);
}
