//! 浏览类端点处理器
#![allow(dead_code)]

use axum::{
    Router,
    routing::get,
    extract::Query,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::error::AppError;
use crate::models::response::{
    SubsonicResponse, ResponseContainer, Indexes, Index,
    Directory, ArtistDetail, AlbumDetail, ArtistResponse, SongResponse, AlbumResponse
};

/// 获取艺术家索引参数
#[derive(Debug, Deserialize)]
pub struct GetIndexesParams {
    pub music_folder_id: Option<i32>,
    pub if_modified_since: Option<i64>,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 获取音乐目录参数
#[derive(Debug, Deserialize)]
pub struct GetMusicDirectoryParams {
    pub id: String,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 获取艺术家参数
#[derive(Debug, Deserialize)]
pub struct GetArtistParams {
    pub id: String,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 获取专辑参数
#[derive(Debug, Deserialize)]
pub struct GetAlbumParams {
    pub id: String,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// 获取歌曲参数
#[derive(Debug, Deserialize)]
pub struct GetSongParams {
    pub id: String,
    pub u: String,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
    pub v: String,
    pub c: String,
    pub f: Option<String>,
}

/// GET /rest/getIndexes
pub async fn get_indexes(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(_params): Query<GetIndexesParams>,
) -> Result<Json<SubsonicResponse<Indexes>>, AppError> {
    // 查询所有艺术家
    let artists = sqlx::query_as::<_, (String, String)>(
        "SELECT id, name FROM artists ORDER BY name"
    )
    .fetch_all(&*pool)
    .await?;

    // 按首字母分组
    let mut index_map: std::collections::HashMap<String, Vec<ArtistResponse>> = std::collections::HashMap::new();

    for (id, name) in artists {
        let first_char = name.chars().next().unwrap_or('#').to_uppercase().to_string();
        let artist = ArtistResponse {
            id,
            name: name.clone(),
            cover_art: None,
            album_count: Some(0), // 这里可以查询专辑数量
        };
        index_map.entry(first_char).or_default().push(artist);
    }

    // 转换为Index结构
    let indexes: Vec<Index> = index_map
        .into_iter()
        .map(|(name, artist)| Index { name, artist })
        .collect();

    let result = Indexes {
        last_modified: chrono::Utc::now().timestamp(),
        indexes,
    };

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: Some(result),
        },
    }))
}

/// GET /rest/getMusicDirectory
pub async fn get_music_directory(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<GetMusicDirectoryParams>,
) -> Result<Json<SubsonicResponse<Directory>>, AppError> {
    // 判断是艺术家还是专辑
    // 如果ID以'a'开头可能是艺术家，以'b'开头可能是专辑
    // 这里简化处理，查询数据库判断

    // 尝试作为专辑查询
    let album = sqlx::query_as::<_, (String, String, String, i32)>(
        "SELECT id, artist_id, name, song_count FROM albums WHERE id = ?"
    )
    .bind(&params.id)
    .fetch_optional(&*pool)
    .await?;

    if let Some((id, artist_id, name, _song_count)) = album {
        // 查询该专辑下的歌曲
        let songs = sqlx::query_as::<_, (String, String)>(
            "SELECT id, title FROM songs WHERE album_id = ? ORDER BY track_number"
        )
        .bind(&id)
        .fetch_all(&*pool)
        .await?;

        let child = songs
            .into_iter()
            .map(|(song_id, title)| crate::models::response::Child {
                id: song_id,
                title,
                is_dir: false,
                artist: None,
                album: None,
                cover_art: None,
                duration: None,
                play_count: None,
            })
            .collect();

        return Ok(Json(SubsonicResponse {
            response: ResponseContainer {
                status: "ok".to_string(),
                version: "1.16.1".to_string(),
                error: None,
                data: Some(Directory {
                    id,
                    name,
                    parent: Some(artist_id),
                    child,
                }),
            },
        }));
    }

    // 尝试作为艺术家查询
    let artist = sqlx::query_as::<_, (String, String)>(
        "SELECT id, name FROM artists WHERE id = ?"
    )
    .bind(&params.id)
    .fetch_optional(&*pool)
    .await?;

    if let Some((id, name)) = artist {
        // 查询该艺术家下的专辑
        let albums = sqlx::query_as::<_, (String, String)>(
            "SELECT id, name FROM albums WHERE artist_id = ? ORDER BY name"
        )
        .bind(&id)
        .fetch_all(&*pool)
        .await?;

        let child = albums
            .into_iter()
            .map(|(album_id, album_name)| crate::models::response::Child {
                id: album_id,
                title: album_name,
                is_dir: true,
                artist: None,
                album: None,
                cover_art: None,
                duration: None,
                play_count: None,
            })
            .collect();

        return Ok(Json(SubsonicResponse {
            response: ResponseContainer {
                status: "ok".to_string(),
                version: "1.16.1".to_string(),
                error: None,
                data: Some(Directory {
                    id,
                    name,
                    parent: None,
                    child,
                }),
            },
        }));
    }

    Err(AppError::not_found("Directory not found"))
}

/// GET /rest/getArtist
pub async fn get_artist(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<GetArtistParams>,
) -> Result<Json<SubsonicResponse<ArtistDetail>>, AppError> {
    // 查询艺术家信息
    let artist = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT id, name, cover_art_path FROM artists WHERE id = ?"
    )
    .bind(&params.id)
    .fetch_optional(&*pool)
    .await?
    .ok_or_else(|| AppError::not_found("ArtistResponse"))?;

    // 查询专辑列表
    let albums = sqlx::query_as::<_, (String, String, Option<i32>, Option<i32>)>(
        "SELECT id, name, year, song_count FROM albums WHERE artist_id = ? ORDER BY year, name"
    )
    .bind(&artist.0)
    .fetch_all(&*pool)
    .await?;

    let album_list = albums
        .into_iter()
        .map(|(id, name, year, song_count)| AlbumResponse {
            id,
            name,
            artist: artist.1.clone(),
            artist_id: Some(artist.0.clone()),
            year,
            cover_art: None,
            song_count,
            created: None,
            duration: None,
            play_count: None,
            genre: None,
        })
        .collect::<Vec<_>>();

    let result = ArtistDetail {
        id: artist.0,
        name: artist.1,
        cover_art: artist.2,
        album_count: album_list.len() as i32,
        album: album_list,
    };

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: Some(result),
        },
    }))
}

/// GET /rest/getAlbum
pub async fn get_album(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<GetAlbumParams>,
) -> Result<Json<SubsonicResponse<AlbumDetail>>, AppError> {
    // 查询专辑信息
    let album = sqlx::query_as::<_, (String, String, String, String, Option<i32>, i32)>(
        "SELECT a.id, a.name, ar.name as artist_name, a.artist_id, a.year, a.song_count
         FROM albums a
         JOIN artists ar ON a.artist_id = ar.id
         WHERE a.id = ?"
    )
    .bind(&params.id)
    .fetch_optional(&*pool)
    .await?
    .ok_or_else(|| AppError::not_found("Album"))?;

    // 查询歌曲列表
    let songs = sqlx::query_as::<_, (String, String, i32, i32, i32, String)>(
        "SELECT id, title, track_number, disc_number, duration, content_type
         FROM songs WHERE album_id = ? ORDER BY disc_number, track_number"
    )
    .bind(&album.0)
    .fetch_all(&*pool)
    .await?;

    // 计算总时长
    let total_duration: i32 = songs.iter().map(|(_, _, _, _, duration, _)| duration).sum();

    let song_list = songs
        .into_iter()
        .map(|(id, title, _track, _disc, duration, content_type)| SongResponse {
            id,
            title,
            artist: album.2.clone(),
            album: album.1.clone(),
            genre: None,
            year: album.4,
            duration,
            bit_rate: None,
            content_type,
            path: None,
            track_number: None,
            disc_number: None,
            cover_art: None,
        })
        .collect();

    let result = AlbumDetail {
        id: album.0,
        name: album.1,
        artist: album.2,
        artist_id: album.3,
        cover_art: None,
        song_count: album.5,
        duration: total_duration,
        song: song_list,
    };

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: Some(result),
        },
    }))
}

/// GET /rest/getSong
pub async fn get_song(
    axum::extract::State(pool): axum::extract::State<Arc<SqlitePool>>,
    Query(params): Query<GetSongParams>,
) -> Result<Json<SubsonicResponse<SongResponse>>, AppError> {
    // 查询歌曲信息
    let song = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<i32>, i32, Option<i32>, String, Option<String>)>(
        "SELECT s.id, s.title, ar.name as artist_name, al.name as album_name,
                s.genre, s.year, s.duration, s.bit_rate, s.content_type, s.file_path
         FROM songs s
         JOIN albums al ON s.album_id = al.id
         JOIN artists ar ON s.artist_id = ar.id
         WHERE s.id = ?"
    )
    .bind(&params.id)
    .fetch_optional(&*pool)
    .await?
    .ok_or_else(|| AppError::not_found("SongResponse"))?;

    let result = SongResponse {
        id: song.0,
        title: song.1,
        artist: song.2,
        album: song.3,
        genre: song.4,
        year: song.5,
        duration: song.6,
        bit_rate: song.7,
        content_type: song.8,
        path: song.9,
        track_number: None,
        disc_number: None,
        cover_art: None,
    };

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: Some(result),
        },
    }))
}

pub fn routes() -> Router<Arc<SqlitePool>> {
    Router::new()
        .route("/rest/getIndexes", get(get_indexes))
        .route("/rest/getMusicDirectory", get(get_music_directory))
        .route("/rest/getArtist", get(get_artist))
        .route("/rest/getAlbum", get(get_album))
        .route("/rest/getSong", get(get_song))
}
