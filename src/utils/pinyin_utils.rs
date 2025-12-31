use std::sync::{
    atomic::{self, AtomicUsize, Ordering},
    Once,
};

use pinyin::ToPinyin;

pub struct Pinyin;

impl Pinyin {
    pub fn new() -> Self {
        Self
    }

    pub fn to_pinyin(&self, text: &str) -> String {
        if text.trim().is_empty() {
            return text.to_string();
        }

        let mut pinyin = String::new();
        for ele in text.to_pinyin() {
            if let Some(pin) = ele {
                pinyin.push_str(&pin.plain());
            }
        }

        if pinyin.is_empty() {
            text.to_string()
        } else {
            pinyin
        }
    }

    pub fn first_char(&self, text: &str) -> String {
        self.to_pinyin(text)
            .chars()
            .next()
            .unwrap_or_default()
            .to_string()
    }
}

#[test]
fn test_pinyin_utils() {
    let pinyin_utils = Pinyin::new();
    let first = pinyin_utils.first_char("我们");
    assert_eq!("w", first.to_lowercase());

    let first = pinyin_utils.first_char("Various");
    println!("first: {}", first);
}
