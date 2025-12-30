use anyhow::{Context, Result};
/// 该工具用于从网络获取音乐元数据
use serde::{Deserialize, Serialize};

use crate::models::response;


const KUGOU_META_URL: &str = "http://mobilecdn.kugou.com/api/v3/search/song?format=json&keyword={}&page=1&pagesize=20&showtype=1";

#[derive(Debug, Clone, Default)]
pub struct MetaClient {
    client: reqwest::Client,
}

impl MetaClient {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn get_kugou_cover(&self, keyword: &str) -> Result<String> {
        let url = KUGOU_META_URL.replace("{}", keyword);
        println!("url: {}", url);

        let res = self.client.get(url).send().await?;
        let json = res.json::<KugouMetaResponseData>().await?;

        // println!("{:?}", json);
        if json.status != 1 {
            return Err(anyhow::anyhow!("status={} not 1", json.status));
        }
        if json.errcode != 0 {
            return Err(anyhow::anyhow!("errcode={} not 0", json.errcode));
        }
        let data = json.data;
        if let Some(infos) = data.info {
            if infos.is_empty() {
                return Err(anyhow::anyhow!("info is empty"));
            }

            // 尝试匹配 album_name
            let match_info = infos.iter().find(|info| {
                if let Some(album_name) = &info.album_name {
                    return album_name.contains(keyword);
                }
                false
            });

            let match_info = if let Some(match_info) = match_info {
                match_info
            } else {
                &infos[0]
            };

            if let Some(trans_param) = &match_info.trans_param {
                if let Some(union_cover) = &trans_param.union_cover {
                    // println!("union_cover: {}", union_cover);
                    let cover_url = union_cover.replace("{size}", "300");
                    // println!("cover_url: {}", cover_url);
                    return Ok(cover_url);
                }
            }
        }

        Err(anyhow::anyhow!("cover not found {}", keyword))
    }

        pub async fn get_kugou_cover_stream(&self, keyword: &str) -> Result<reqwest::Response> {
        let url = self.get_kugou_cover(keyword).await?;

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to stream from Kugou server")?;

        Ok(response)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct KugouMetaResponseData {
    status: i32,
    errcode: i32,
    data: KugouInfo,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct KugouInfo {
    info: Option<Vec<KugouSongInfo>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct KugouSongInfo {
    trans_param: Option<TransParam>,
    album_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TransParam {
    union_cover: Option<String>,
}
