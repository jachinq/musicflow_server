

// cargo run --example test_cover_art
fn main() {
    let cover_art_id = "al-ce60a8a2-a40f-43ef-ad26-55051da8999f";

    // 如果是这种格式的 al-ce60a8a2-a40f-43ef-ad26-55051da8999f，提取出 al-ce60a8a2
    let cover_art_id = if cover_art_id.starts_with("al-") {
        let a = cover_art_id
            .split('-')
            .nth(1)
            .unwrap_or_default()
            .to_string();
        &format!("al-{}", a.clone()) // 取第二个元素
                                  // tracing::warn!("Cover art ID is in old format: {}", cover_art_id);
    } else {
        cover_art_id
    };
    println!("Cover art ID is: {}", cover_art_id);

    let size = Some(5);
    let final_size = size.unwrap_or(300).max(50).min(2000) as u32;
    println!("size={:?} final_size={}", size, final_size);

}
