use anyhow::Result;
use musicflow_server::utils::MetaClient;

#[tokio::main]
/// cargo run --example kugou_meta
async fn main() -> Result<()> {
    get_kugou_album_cover().await;
    get_kugou_artist_cover().await;
    // parse_json()?;
    Ok(())
}

fn _parse_json() -> Result<()> {
    let text = r#"
    {"data":{"peripherycount":0,"songcount":54,"singername":"陈冠蒲","profile":"陈冠蒲，1970年11月18日出生于中国台湾省屏东市，新加坡籍华人，华语男歌手。陈冠蒲为多部电视剧主唱主题曲，广为人知。还曾被誉为史上最HIGH最温柔的男高音，跨越男人的洒脱与女性的柔美，初听给人以女性的错觉。陈冠蒲生长于台湾南部的屏东， 以相信简单、崇尚自然为人生信条。代表作品有专辑《就让你走》，歌曲《别说》《太多》《蓝眼泪》等。","style":1,"mvcount":29,"grade":4,"singerid":460,"is_settled_author":0,"albumcount":9,"mix_intro":"","classical_work_total":0,"has_long_intro":1,"charge_video_count":0,"alias":"","identity":1,"intro":"陈冠蒲，1970年11月18日出生于中国台湾省屏东市，新加坡籍华人，华语男歌手。陈冠蒲为多部电视剧主唱主题曲，广为人知。还曾被誉为史上最HIGH最温柔的男高音，跨越男人的洒脱与女性的柔美 ，初听给人以女性的错觉。陈冠蒲生长于台湾南部的屏东，以相信简单、崇尚自然为人生信条。代表作品有专辑《就让你走》，歌曲《别说》《太多》《蓝眼泪》等。",
"imgurl":"http:\/\/singerimg.kugou.com\/uploadpic\/softhead\/{size}\/20250331\/20250331114201311261.jpg"},"errcode":0,"status":1,"error":""}
    "#;

    let text = text.trim().replace(" ", "");

    let json: serde_json::Value = serde_json::from_str(&text)?;
    println!("{:?}", json);

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
    let keyword = "陈冠蒲";
    let client = MetaClient::new();
    let result = client.get_kugou_artist_cover(keyword).await;
    if let Ok(cover_url) = result {
        println!("{}", cover_url);
    }
}
