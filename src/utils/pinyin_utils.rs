use std::sync::{
    Once, atomic::{self, AtomicUsize, Ordering}
};

/// 懒加载 mandarin_to_pinyin::init_map()
/// 避免多次初始化导致性能问题
static INIT: Once = Once::new();
static INIT_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct Pinyin;

impl Pinyin {
    pub fn new() -> Self {
        INIT.call_once(|| {
            println!("start init pinyin map");
            INIT_COUNT.fetch_add(1, Ordering::SeqCst);
            let _ = mandarin_to_pinyin::init_map(None);
        });

        Self
    }

    pub fn to_pinyin(&self, text: &str) -> String {
        if text.trim().is_empty() {
            return text.to_string();
        }

        match mandarin_to_pinyin::to_pinyin_string(text, " ") {
            Ok(pinyin) => pinyin,
            Err(_) => text.to_string(),
        }
    }

    pub fn first_char(&self, text: &str) -> String {
        self.to_pinyin(text).chars().next().unwrap_or_default().to_string()
    }
}

#[test]
fn test_pinyin_utils() {
    let pinyin_utils = Pinyin::new();
    let first = pinyin_utils.first_char("我们");
    assert_eq!("w", first.to_lowercase());
}

#[test]
fn test_pinyin_init_once() {
    let _ = Pinyin::new();
    let _ = Pinyin::new();
    let _ = Pinyin::new();
    let _ = Pinyin::new();
    let _ = Pinyin::new();
    assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 1);
}
