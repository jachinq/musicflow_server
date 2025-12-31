use anyhow::Result;
use musicflow_server::utils::MetaClient;

#[tokio::main]
async fn main() -> Result<()> {
    get_kugou_album_cover().await;
    get_kugou_artist_cover().await;
    Ok(())
}

async fn get_kugou_album_cover() {
    let keyword = "光辉岁月十五年";
    let client = MetaClient::new();
    let result = client.get_kugou_album_cover(keyword).await;
    if let Ok(cover_url) = result {
        println!("{}", cover_url);
    }
}

async fn get_kugou_artist_cover() {
    let keyword = "徐良";
    let client = MetaClient::new();
    let result = client.get_kugou_album_cover(keyword).await;
    if let Ok(cover_url) = result {
        println!("{}", cover_url);
    }
}