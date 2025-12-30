//! 搜索端点处理器
#![allow(dead_code)]

use crate::error::AppError;
use crate::extractors::Format;
use crate::models::dto::{AlbumDetailDto, AlbumDto, ArtistDto, SongDetailDto, SongDto};
use crate::models::response::{
    AlbumResponse, ArtistResponse, SearchResult, SearchResult2, SearchResult2Response,
    SearchResult3, SearchResult3Response, SearchResultResponse, Song,
};
use crate::response::ApiResponse;
use crate::services::song_service::CommState;
use axum::{extract::Query, routing::get, Router};
use serde::Deserialize;

/// 搜索参数 (search3)
#[derive(Debug, Deserialize)]
pub struct Search3Params {
    pub query: String,
    pub artist_count: Option<i32>,
    pub artist_offset: Option<i32>,
    pub album_count: Option<i32>,
    pub album_offset: Option<i32>,
    pub song_count: Option<i32>,
    pub song_offset: Option<i32>,
}

/// 搜索参数 (search2)
#[derive(Debug, Deserialize)]
pub struct Search2Params {
    pub query: String,
    pub artist_count: Option<i32>,
    pub artist_offset: Option<i32>,
    pub album_count: Option<i32>,
    pub album_offset: Option<i32>,
    pub song_count: Option<i32>,
    pub song_offset: Option<i32>,
}

/// 搜索参数 (search)
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub any: Option<String>,
    pub count: Option<i32>,
    pub offset: Option<i32>,
}

/// GET /rest/search3
pub async fn search3(
    axum::extract::State(state): axum::extract::State<CommState>,
    Query(params): Query<Search3Params>,
    Format(format): Format,
) -> Result<ApiResponse<SearchResult3Response>, AppError> {
    let artist_count = params.artist_count.unwrap_or(20);
    let artist_offset = params.artist_offset.unwrap_or(0);
    let album_count = params.album_count.unwrap_or(20);
    let album_offset = params.album_offset.unwrap_or(0);
    let song_count = params.song_count.unwrap_or(20);
    let song_offset = params.song_offset.unwrap_or(0);

    // 搜索艺术家 - 使用 DTO
    let artist_dtos = sqlx::query_as::<_, ArtistDto>(
        "SELECT id, name FROM artists
         WHERE name LIKE ?
         ORDER BY name
         LIMIT ? OFFSET ?",
    )
    .bind(format!("%{}%", params.query))
    .bind(artist_count)
    .bind(artist_offset)
    .fetch_all(&*state.pool)
    .await?;

    let artists: Vec<ArtistResponse> = artist_dtos.into_iter().map(Into::into).collect();

    // 搜索专辑 - 使用 DTO
    let album_dtos = sqlx::query_as::<_, AlbumDetailDto>(
        "SELECT a.id, a.name, ar.name as artist, a.artist_id, a.year, a.genre,
                    a.cover_art_path, a.song_count, a.duration, a.play_count
             FROM albums a
             JOIN artists ar ON a.artist_id = ar.id
         WHERE a.name LIKE ? OR ar.name LIKE ?
         ORDER BY a.name
         LIMIT ? OFFSET ?",
    )
    .bind(format!("%{}%", params.query))
    .bind(format!("%{}%", params.query))
    .bind(album_count)
    .bind(album_offset)
    .fetch_all(&*state.pool)
    .await?;

    let albums: Vec<AlbumResponse> = album_dtos.into_iter().map(Into::into).collect();

    // 搜索歌曲 - 使用 DTO
    let song_dtos = sqlx::query_as::<_, SongDetailDto>(&format!(
        "{}
         WHERE s.title LIKE ? OR al.name LIKE ? OR ar.name LIKE ?
         ORDER BY s.title
         LIMIT ? OFFSET ?",
        state.song_service.detail_sql()
    ))
    .bind(format!("%{}%", params.query))
    .bind(format!("%{}%", params.query))
    .bind(format!("%{}%", params.query))
    .bind(song_count)
    .bind(song_offset)
    .fetch_all(&*state.pool)
    .await?;

    let songs: Vec<Song> = song_dtos.into_iter().map(Into::into).collect();

    let result = SearchResult3Response {
        search_result3: SearchResult3 {
            artist: artists,
            album: albums,
            song: songs,
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/search2
pub async fn search2(
    axum::extract::State(state): axum::extract::State<CommState>,
    Query(params): Query<Search2Params>,
    Format(format): Format,
) -> Result<ApiResponse<SearchResult2Response>, AppError> {
    let artist_count = params.artist_count.unwrap_or(20);
    let artist_offset = params.artist_offset.unwrap_or(0);
    let album_count = params.album_count.unwrap_or(20);
    let album_offset = params.album_offset.unwrap_or(0);
    let song_count = params.song_count.unwrap_or(20);
    let song_offset = params.song_offset.unwrap_or(0);

    // 搜索艺术家 - 使用 DTO
    let artist_dtos = sqlx::query_as::<_, ArtistDto>(
        "SELECT id, name FROM artists
         WHERE name LIKE ?
         ORDER BY name
         LIMIT ? OFFSET ?",
    )
    .bind(format!("%{}%", params.query))
    .bind(artist_count)
    .bind(artist_offset)
    .fetch_all(&*state.pool)
    .await?;

    let artists: Vec<ArtistResponse> = artist_dtos.into_iter().map(Into::into).collect();

    // 搜索专辑 - 使用 DTO
    let album_dtos = sqlx::query_as::<_, AlbumDto>(
        "SELECT a.id, a.name, ar.name as artist, a.year, a.song_count
         FROM albums a
         JOIN artists ar ON a.artist_id = ar.id
         WHERE a.name LIKE ? OR ar.name LIKE ?
         ORDER BY a.name
         LIMIT ? OFFSET ?",
    )
    .bind(format!("%{}%", params.query))
    .bind(format!("%{}%", params.query))
    .bind(album_count)
    .bind(album_offset)
    .fetch_all(&*state.pool)
    .await?;

    let albums: Vec<AlbumResponse> = album_dtos.into_iter().map(Into::into).collect();

    // 搜索歌曲 - 使用 DTO
    let song_dtos = sqlx::query_as::<_, SongDto>(
        "SELECT s.id, s.title, ar.name as artist, al.name as album, s.duration, s.content_type
         FROM songs s
         JOIN albums al ON s.album_id = al.id
         JOIN artists ar ON s.artist_id = ar.id
         WHERE s.title LIKE ? OR al.name LIKE ? OR ar.name LIKE ?
         ORDER BY s.title
         LIMIT ? OFFSET ?",
    )
    .bind(format!("%{}%", params.query))
    .bind(format!("%{}%", params.query))
    .bind(format!("%{}%", params.query))
    .bind(song_count)
    .bind(song_offset)
    .fetch_all(&*state.pool)
    .await?;

    let songs: Vec<Song> = song_dtos.into_iter().map(Into::into).collect();

    let result = SearchResult2Response {
        search_result2: SearchResult2 {
            artist: artists,
            album: albums,
            song: songs,
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/search
pub async fn search(
    axum::extract::State(state): axum::extract::State<CommState>,
    Query(params): Query<SearchParams>,
    Format(format): Format,
) -> Result<ApiResponse<SearchResultResponse>, AppError> {
    let count = params.count.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    // 构建查询条件
    let mut conditions = Vec::new();
    let mut query_params = Vec::new();

    if let Some(artist) = &params.artist {
        conditions.push("ar.name LIKE ?");
        query_params.push(format!("%{}%", artist));
    }
    if let Some(album) = &params.album {
        conditions.push("al.name LIKE ?");
        query_params.push(format!("%{}%", album));
    }
    if let Some(title) = &params.title {
        conditions.push("s.title LIKE ?");
        query_params.push(format!("%{}%", title));
    }
    if let Some(any) = &params.any {
        conditions.push("(s.title LIKE ? OR al.name LIKE ? OR ar.name LIKE ?)");
        query_params.push(format!("%{}%", any));
        query_params.push(format!("%{}%", any));
        query_params.push(format!("%{}%", any));
    }

    if conditions.is_empty() {
        return Err(AppError::missing_parameter("Search criteria"));
    }

    let _where_clause = conditions.join(" AND ");

    // 搜索艺术家 - 使用 DTO
    let artists = if params.artist.is_some() || params.any.is_some() {
        let query = format!(
            "SELECT DISTINCT ar.id, ar.name
             FROM artists ar
             {}
             ORDER BY ar.name
             LIMIT ? OFFSET ?",
            if params.any.is_some() {
                "WHERE ar.name LIKE ?"
            } else {
                ""
            }
        );
        let mut query_builder = sqlx::query_as::<_, ArtistDto>(&query);
        for param in &query_params {
            query_builder = query_builder.bind(param);
        }
        query_builder = query_builder.bind(count).bind(offset);
        let dtos = query_builder.fetch_all(&*state.pool).await?;
        dtos.into_iter().map(Into::into).collect()
    } else {
        vec![]
    };

    // 搜索专辑 - 使用 DTO
    let albums = if params.album.is_some() || params.any.is_some() {
        let query = format!(
            "SELECT DISTINCT al.id, al.name, ar.name as artist, al.year, al.song_count
             FROM albums al
             JOIN artists ar ON al.artist_id = ar.id
             WHERE {}
             ORDER BY al.name
             LIMIT ? OFFSET ?",
            if params.any.is_some() {
                "(al.name LIKE ? OR ar.name LIKE ?)"
            } else {
                "al.name LIKE ?"
            }
        );
        let mut query_builder = sqlx::query_as::<_, AlbumDto>(&query);
        for param in &query_params {
            query_builder = query_builder.bind(param);
        }
        if params.any.is_some() {
            for param in &query_params {
                query_builder = query_builder.bind(param);
            }
        }
        query_builder = query_builder.bind(count).bind(offset);
        let dtos = query_builder.fetch_all(&*state.pool).await?;
        dtos.into_iter().map(Into::into).collect()
    } else {
        vec![]
    };

    // 搜索歌曲 - 使用 DTO
    let songs = if params.title.is_some() || params.any.is_some() {
        let query = format!(
            "SELECT DISTINCT s.id, s.title, ar.name as artist, al.name as album, s.duration, s.content_type
             FROM songs s
             JOIN albums al ON s.album_id = al.id
             JOIN artists ar ON s.artist_id = ar.id
             WHERE {}
             ORDER BY s.title
             LIMIT ? OFFSET ?",
            if params.any.is_some() {
                "(s.title LIKE ? OR al.name LIKE ? OR ar.name LIKE ?)"
            } else {
                "s.title LIKE ?"
            }
        );
        let mut query_builder = sqlx::query_as::<_, SongDto>(&query);
        for param in &query_params {
            query_builder = query_builder.bind(param);
        }
        if params.any.is_some() {
            for param in &query_params {
                query_builder = query_builder.bind(param);
            }
            for param in &query_params {
                query_builder = query_builder.bind(param);
            }
        }
        query_builder = query_builder.bind(count).bind(offset);
        let dtos = query_builder.fetch_all(&*state.pool).await?;
        dtos.into_iter().map(Into::into).collect()
    } else {
        vec![]
    };

    let result = SearchResultResponse {
        search_result: SearchResult {
            artist: artists,
            album: albums,
            song: songs,
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

pub fn routes() -> Router<CommState> {
    Router::new()
        .route("/rest/search3", get(search3))
        .route("/rest/search2", get(search2))
        .route("/rest/search", get(search))
}
