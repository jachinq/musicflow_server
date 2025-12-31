#![allow(dead_code)]
use std::fmt::Debug;

use anyhow::{Context, Result};
/// 该工具用于从网络获取音乐元数据
use serde::{Deserialize, Serialize};

use crate::models::response;

const DEFAULT_SIZE : &str = "600";
const KUGOU_META_URL: &str = "http://mobilecdn.kugou.com/api/v3/search/song?format=json&keyword={}&page=1&pagesize=20&showtype=1";
const KUGOU_ALBUM_URL: &str = "http://mobilecdn.kugou.com/api/v3/search/album?keyword={}";
const KUGOU_ARTISTS_URL: &str =
    "http://mobilecdn.kugou.com/api/v3/search/singer?keyword={}&page=1&pagesize=20";
const KUGOU_ARTIST_URL: &str =
    "http://mobilecdn.kugou.com/api/v3/singer/info?singerid={}&with_res_tag=1";

#[derive(Debug, Clone, Default)]
pub struct MetaClient {
    client: reqwest::Client,
}

impl MetaClient {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 kugou 获取专辑封面 url
    pub async fn get_kugou_album_cover(&self, keyword: &str) -> Result<String> {
        let url = KUGOU_ALBUM_URL.replace("{}", keyword);
        println!("url: {}", url);

        let res = self.client.get(url).send().await?;
        let json = res
            .json::<KugouResponseData<KugouInfoWrapper<KugouAlbum>>>()
            .await?;

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
                if let Some(album_name) = &info.albumname {
                    return album_name.contains(keyword);
                }
                false
            });

            let match_info = if let Some(match_info) = match_info {
                match_info
            } else {
                &infos[0]
            };

            if let Some(union_cover) = &match_info.imgurl {
                let cover_url = union_cover.replace("{size}", DEFAULT_SIZE);
                return Ok(cover_url);
            }
        }

        Err(anyhow::anyhow!("cover not found {}", keyword))
    }

    /// 从 kugou 获取专辑封面流
    pub async fn get_kugou_album_cover_stream(&self, keyword: &str) -> Result<reqwest::Response> {
        let url = self.get_kugou_album_cover(keyword).await?;

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to stream from Kugou server")?;

        Ok(response)
    }

    /// 从 kugou 获取歌手 id
    pub async fn get_kugou_artist_id(&self, keyword: &str) -> Result<i32> {
        let url = KUGOU_ARTISTS_URL.replace("{}", keyword);
        println!("url: {}", url);

        let res = self.client.get(url).send().await?;
        let json = res.json::<KugouResponseDatas<KugouArtists>>().await?;

        // println!("{:?}", json);
        if json.status != 1 {
            return Err(anyhow::anyhow!("status={} not 1", json.status));
        }
        if json.errcode != 0 {
            return Err(anyhow::anyhow!("errcode={} not 0", json.errcode));
        }
        let data = json.data;
        if data.is_empty() {
            return Err(anyhow::anyhow!("data is empty"));
        }

        // 尝试匹配 singername
        let match_info = data.iter().find(|info| {
            if let Some(album_name) = &info.singername {
                return album_name.contains(keyword);
            }
            false
        });

        let match_info = if let Some(match_info) = match_info {
            match_info
        } else {
            &data[0]
        };
        if let Some(singerid) = &match_info.singerid {
            return Ok(*singerid);
        }

        Err(anyhow::anyhow!("singer id not found {}", keyword))
    }

    /// 从 kugou 获取歌手封面 url
    pub async fn get_kugou_artist_cover(&self, keyword: &str) -> Result<String> {
        // 需要先拿到歌手 id
        let singerid = self.get_kugou_artist_id(keyword).await?;
        let url = KUGOU_ARTIST_URL.replace("{}", &singerid.to_string());
        println!("url: {}", url);

        let res = self.client.get(url).send().await?;
        let json = res.json::<KugouResponseData<KugouArtist>>().await?;

        // println!("{:?}", json);
        if json.status != 1 {
            return Err(anyhow::anyhow!("status={} not 1", json.status));
        }
        if json.errcode != 0 {
            return Err(anyhow::anyhow!("errcode={} not 0", json.errcode));
        }
        let data = json.data;

        if let Some(imgurl) = &data.imgurl {
            let cover_url = imgurl.replace("{size}", DEFAULT_SIZE);
            return Ok(cover_url);
        }
        Err(anyhow::anyhow!("artist cover not found {}", keyword))
    }

    /// 从 kugou 获取歌手封面流
    pub async fn get_kugou_artist_cover_stream(&self, keyword: &str) -> Result<reqwest::Response> {
        let url = self.get_kugou_artist_cover(keyword).await?;

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to stream from Kugou server")?;

        Ok(response)
    }
}

/// 返回的 data 是单个对象
#[derive(Debug, Deserialize, Serialize, Clone)]
struct KugouResponseData<T> {
    status: i32,
    errcode: i32,
    data: T,
}

/// 返回的 data 是列表
#[derive(Debug, Deserialize, Serialize, Clone)]
struct KugouResponseDatas<T> {
    status: i32,
    errcode: i32,
    data: Vec<T>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct KugouInfoWrapper<T> {
    info: Option<Vec<T>>,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
struct KugouArtists {
    singername: Option<String>,
    singerid: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct KugouArtist {
    singername: Option<String>,
    singerid: Option<i32>,
    imgurl: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct KugouAlbum {
    albumname: Option<String>,
    imgurl: Option<String>,
}