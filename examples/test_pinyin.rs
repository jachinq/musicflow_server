/// cargo run --example test_pinyin
fn main() {
  let _ = mandarin_to_pinyin::init_map(None);
  let pinyin = mandarin_to_pinyin::to_pinyin_string("张芸京", " ");
  println!("pinyin: {:?}", pinyin);
}