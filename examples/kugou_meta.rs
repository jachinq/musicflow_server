use anyhow::Result;
use musicflow_server::utils::MetaClient;

#[tokio::main]
async fn main() -> Result<()> {
    let keyword = "光辉岁月十五年";
    let client = MetaClient::new();
    let result = client.get_kugou_cover(keyword).await;
    if let Ok(cover_url) = result {
        println!("{}", cover_url);
    }
    Ok(())
}
