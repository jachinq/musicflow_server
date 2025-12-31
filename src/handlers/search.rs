//! 搜索端点处理器
#![allow(dead_code)]

use crate::error::AppError;
use crate::extractors::Format;
use crate::models::response::{
    SearchResult2, SearchResult2Response, SearchResult3, SearchResult3Response,
    SearchResultResponse,
};
use crate::response::ApiResponse;
use crate::services::search_service::SearchParams;
use crate::services::SearchService;
use axum::{extract::Query, routing::get, Router};
use serde::Deserialize;
use std::sync::Arc;

/// 搜索参数 (search3)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct Search2Params {
    pub query: String,
    pub artist_count: Option<i32>,
    pub artist_offset: Option<i32>,
    pub album_count: Option<i32>,
    pub album_offset: Option<i32>,
    pub song_count: Option<i32>,
    pub song_offset: Option<i32>,
}

/// GET /rest/search3
pub async fn search3(
    axum::extract::State(state): axum::extract::State<Arc<SearchService>>,
    Query(params): Query<Search3Params>,
    Format(format): Format,
) -> Result<ApiResponse<SearchResult3Response>, AppError> {
    let artist_count = params.artist_count.unwrap_or(20);
    let artist_offset = params.artist_offset.unwrap_or(0);
    let album_count = params.album_count.unwrap_or(20);
    let album_offset = params.album_offset.unwrap_or(0);
    let song_count = params.song_count.unwrap_or(20);
    let song_offset = params.song_offset.unwrap_or(0);

    let query = if params.query.eq("\"\"") {
        ""
    } else {
        &params.query
    };

    let params = SearchParams {
        query: query.to_string(),
        artist_count,
        artist_offset,
        album_count,
        album_offset,
        song_count,
        song_offset,
    };

    let result = state.search_all(params).await?;

    let result = SearchResult3Response {
        search_result3: SearchResult3 {
            artist: result.artists.into_iter().map(Into::into).collect(),
            album: result.albums.into_iter().map(Into::into).collect(),
            song: result.songs.into_iter().map(Into::into).collect(),
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/search2
pub async fn search2(
    axum::extract::State(state): axum::extract::State<Arc<SearchService>>,
    Query(params): Query<Search2Params>,
    Format(format): Format,
) -> Result<ApiResponse<SearchResult2Response>, AppError> {
    let artist_count = params.artist_count.unwrap_or(20);
    let artist_offset = params.artist_offset.unwrap_or(0);
    let album_count = params.album_count.unwrap_or(20);
    let album_offset = params.album_offset.unwrap_or(0);
    let song_count = params.song_count.unwrap_or(20);
    let song_offset = params.song_offset.unwrap_or(0);

    let params = SearchParams {
        query: params.query,
        artist_count,
        artist_offset,
        album_count,
        album_offset,
        song_count,
        song_offset,
    };

    let result = state.search_all_simple(params).await?;

    let result = SearchResult2Response {
        search_result2: SearchResult2 {
            artist: result.artists.into_iter().map(Into::into).collect(),
            album: result.albums.into_iter().map(Into::into).collect(),
            song: result.songs.into_iter().map(Into::into).collect(),
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/search
pub async fn search(Format(format): Format) -> Result<ApiResponse<SearchResultResponse>, AppError> {
    // TODO: 实现搜索功能
    Ok(ApiResponse::ok(None, format))
}

pub fn routes() -> Router<Arc<SearchService>> {
    Router::new()
        .route("/rest/search3", get(search3))
        .route("/rest/search2", get(search2))
        .route("/rest/search", get(search))
}
