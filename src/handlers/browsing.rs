//! 浏览类端点处理器
#![allow(dead_code)]

use crate::error::AppError;
use crate::extractors::Format;
use crate::models::response::{
    AlbumDetail, AlbumDetailResponse, AlbumList2, AlbumList2Response, AlbumResponse, ArtistDetail,
    ArtistDetailResponse, ArtistIndex, ArtistResponse, Artists, ArtistsResponse, Directory, Genre,
    Genres, GenresResponse, Index, Indexes, RandomSongs, RandomSongsResponse, Song, SongResponse,
    SongsByGenreResponse, SongsResponse, TopSongs, TopSongsResponse,
};
use crate::response::ApiResponse;
use crate::services::browsing_service::AlbumListType;
use crate::services::BrowsingService;
use crate::utils::Pinyin;
use axum::{extract::Query, routing::get, Router};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

/// 获取艺术家列表参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetArtistsParams {
    pub music_folder_id: Option<i32>,
}
/// 获取艺术家索引参数
#[derive(Debug, Deserialize)]
pub struct GetIndexesParams {
    pub music_folder_id: Option<i32>,
    pub if_modified_since: Option<i64>,
}

/// 获取音乐目录参数
#[derive(Debug, Deserialize)]
pub struct GetMusicDirectoryParams {
    pub id: String,
}

/// 获取艺术家参数
#[derive(Debug, Deserialize)]
pub struct GetArtistParams {
    pub id: String,
}

/// 获取专辑参数
#[derive(Debug, Deserialize)]
pub struct GetAlbumParams {
    pub id: String,
}

/// 获取歌曲参数
#[derive(Debug, Deserialize)]
pub struct GetSongParams {
    pub id: String,
}

/// GET /rest/getIndexes
pub async fn get_indexes(
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
    Query(_params): Query<GetIndexesParams>,
) -> Result<ApiResponse<Indexes>, AppError> {
    let artists = state.browseing_service.get_artist_indexes().await?;

    // 按首字母分组
    let mut index_map: std::collections::HashMap<String, Vec<ArtistResponse>> =
        std::collections::HashMap::new();

    for artist in artists {
        let first_char = Pinyin::new().first_char(&artist.name).to_uppercase();
        let artist = ArtistResponse {
            cover_art: Some(format!("ar-{}", artist.id)),
            id: artist.id,
            name: artist.name,
            album_count: Some(0), // TODO 这里可以查询专辑数量
        };
        index_map.entry(first_char).or_default().push(artist);
    }

    // 转换为Index结构
    let mut indexes: Vec<Index> = index_map
        .into_iter()
        .map(|(name, artist)| Index { name, artist })
        .collect();
    // 按 name 排序
    indexes.sort_by(|a, b| a.name.cmp(&b.name));

    let result = Indexes {
        last_modified: chrono::Utc::now().timestamp(),
        indexes,
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getMusicDirectory
pub async fn get_music_directory(
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
    Query(params): Query<GetMusicDirectoryParams>,
) -> Result<ApiResponse<Directory>, AppError> {
    // 判断是艺术家还是专辑
    // 如果ID以'a'开头可能是艺术家，以'b'开头可能是专辑
    // 这里简化处理，查询数据库判断

    // 尝试作为专辑查询
    let album = sqlx::query_as::<_, (String, String, String, i32)>(
        "SELECT id, artist_id, name, song_count FROM albums WHERE id = ?",
    )
    .bind(&params.id)
    .fetch_optional(&*state.pool)
    .await?;

    if let Some((id, artist_id, name, _song_count)) = album {
        // 查询该专辑下的歌曲
        let songs = sqlx::query_as::<_, (String, String)>(
            "SELECT id, title FROM songs WHERE album_id = ? ORDER BY track_number",
        )
        .bind(&id)
        .fetch_all(&*state.pool)
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

        return Ok(ApiResponse::ok(
            Some(Directory {
                id,
                name,
                parent: Some(artist_id),
                child,
            }),
            format,
        ));
    }

    // 尝试作为艺术家查询
    let artist = sqlx::query_as::<_, (String, String)>("SELECT id, name FROM artists WHERE id = ?")
        .bind(&params.id)
        .fetch_optional(&*state.pool)
        .await?;

    if let Some((id, name)) = artist {
        // 查询该艺术家下的专辑
        let albums = sqlx::query_as::<_, (String, String)>(
            "SELECT id, name FROM albums WHERE artist_id = ? ORDER BY name",
        )
        .bind(&id)
        .fetch_all(&*state.pool)
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

        return Ok(ApiResponse::ok(
            Some(Directory {
                id,
                name,
                parent: None,
                child,
            }),
            format,
        ));
    }

    Err(AppError::not_found("Directory not found"))
}

/// GET /rest/getArtists
pub async fn get_artists(
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
    Query(_params): Query<GetArtistsParams>,
) -> Result<ApiResponse<ArtistsResponse>, AppError> {
    // 查询艺术家信息
    let artist = state.browseing_service.get_artist_indexes().await?;

    let mut index_map: HashMap<String, Vec<ArtistResponse>> = HashMap::new();
    artist.into_iter().for_each(|a| {
        let first_char = Pinyin::new().first_char(&a.name).to_uppercase();
        index_map
            .entry(first_char)
            .or_default()
            .push(ArtistResponse::from(a));
    });

    let indexs = index_map
        .keys()
        .map(|k| {
            let list = index_map.get(k);
            let list = list.cloned();
            ArtistIndex {
                name: k.to_string(),
                artist: list.unwrap_or_default(),
            }
        })
        .collect();

    let result = ArtistsResponse {
        artists: Artists { index: indexs },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getArtist
pub async fn get_artist(
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
    Query(params): Query<GetArtistParams>,
) -> Result<ApiResponse<ArtistDetailResponse>, AppError> {
    let (artist, album_list) = state.browseing_service.get_artist(&params.id).await?;

    let result = ArtistDetailResponse {
        artist: ArtistDetail {
            cover_art: Some(format!("ar-{}", artist.id)),
            id: artist.id,
            name: artist.name,
            album_count: album_list.len() as i32,
            album: AlbumResponse::from_dto_details(album_list),
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getAlbum
pub async fn get_album(
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
    Query(params): Query<GetAlbumParams>,
) -> Result<ApiResponse<AlbumDetailResponse>, AppError> {
    let (album, songs) = state.browseing_service.get_album(&params.id).await?;
    // 计算总时长
    let total_duration: i32 = songs.iter().map(|s| s.duration).sum();
    // tracing::info!("al = {:?}", album);

    let song_list = songs.into_iter().map(|s| s.into()).collect();

    let result = AlbumDetailResponse {
        album: AlbumDetail {
            id: album.id,
            name: album.name,
            artist: album.artist,
            artist_id: album.artist_id,
            cover_art: album.cover_art_path,
            song_count: album.song_count,
            duration: total_duration,
            song: song_list,
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getSong
pub async fn get_song(
    claims: crate::middleware::auth_middleware::Claims,
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
    Query(params): Query<GetSongParams>,
) -> Result<ApiResponse<SongResponse>, AppError> {
    let user_id = claims.sub;
    // 查询歌曲信息
    let song = state
        .browseing_service
        .get_song(&user_id, &params.id)
        .await?;

    let result = SongResponse { song: song.into() };

    Ok(ApiResponse::ok(Some(result), format))
}

/// 获取专辑列表参数
#[derive(Debug, Deserialize)]
pub struct GetAlbumListParams {
    pub r#type: String, // random, newest, highest, frequent, recent, starred, alphabetical
    pub size: Option<i32>,
    pub offset: Option<i32>,
    pub from_year: Option<i32>,
    pub to_year: Option<i32>,
    pub genre: Option<String>,
    pub music_folder_id: Option<String>,
}

/// 获取随机歌曲参数
#[derive(Debug, Deserialize)]
pub struct GetRandomSongsParams {
    pub size: Option<i32>,
    pub genre: Option<String>,
    pub from_year: Option<i32>,
    pub to_year: Option<i32>,
    pub music_folder_id: Option<String>,
}

/// 获取艺术家信息参数
#[derive(Debug, Deserialize)]
pub struct GetArtistInfoParams {
    pub id: String,
    pub count: Option<i32>,
    pub include_not_present: Option<bool>,
}

/// GET /rest/getAlbumList2 - 获取专辑列表 (v2)
pub async fn get_album_list2(
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
    Query(params): Query<GetAlbumListParams>,
) -> Result<ApiResponse<AlbumList2Response>, AppError> {
    let size = params.size.unwrap_or(10).min(500); // 限制最大500
    let offset = params.offset.unwrap_or(0);
    let albums = state
        .browseing_service
        .get_album_list(
            AlbumListType::from_str(&params.r#type).unwrap_or_default(),
            size,
            offset,
        )
        .await?;

    let album_responses = AlbumResponse::from_dto_details(albums);

    let result = AlbumList2Response {
        album_list2: AlbumList2 {
            albums: album_responses,
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getRandomSongs - 获取随机歌曲
pub async fn get_random_songs(
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
    Query(params): Query<GetRandomSongsParams>,
) -> Result<ApiResponse<RandomSongsResponse>, AppError> {
    let size = params.size.unwrap_or(10).min(500);
    let songs = state
        .browseing_service
        .get_random_songs(
            size,
            params.genre.as_deref(),
            params.from_year,
            params.to_year,
        )
        .await?;

    let song_responses = Song::from_detail_dtos(songs);

    let result = RandomSongsResponse {
        random_songs: RandomSongs {
            song: song_responses,
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getArtistInfo - 获取艺术家信息
pub async fn get_artist_info(
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
    Query(params): Query<GetArtistInfoParams>,
) -> Result<ApiResponse<crate::models::response::ArtistInfo>, AppError> {
    // let artist = state.browseing_service.get_artist_info(&params.id, params.count, params.include_not_present).await?;

    // 查询艺术家基本信息
    let artist = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT name, music_brainz_id FROM artists WHERE id = ?",
    )
    .bind(&params.id)
    .fetch_optional(&*state.pool)
    .await?;

    if artist.is_none() {
        return Err(AppError::not_found("Artist"));
    }

    // 这里可以集成外部API (如 Last.fm) 获取更多信息
    // 目前返回基本信息
    let result = crate::models::response::ArtistInfo {
        biography: None,
        music_brainz_id: artist.and_then(|(_, mbid)| mbid),
        last_fm_url: None,
        similar_artists: None,
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getArtistInfo2 - 获取艺术家信息 (v2)
pub async fn get_artist_info2(
    format: Format,
    state: axum::extract::State<BrowsingState>,
    params: Query<GetArtistInfoParams>,
) -> Result<ApiResponse<crate::models::response::ArtistInfo>, AppError> {
    // ArtistInfo2 与 ArtistInfo 结构相同
    get_artist_info(format, state, params).await
}

/// 获取热门歌曲参数
#[derive(Debug, Deserialize)]
pub struct GetTopSongsParams {
    pub artist: String,
    pub count: Option<i32>,
}

/// 获取流派歌曲参数
#[derive(Debug, Deserialize)]
pub struct GetSongsByGenreParams {
    pub genre: String,
    pub count: Option<i32>,
    pub offset: Option<i32>,
    pub music_folder_id: Option<String>,
}

/// GET /rest/getTopSongs - 获取艺术家热门歌曲
pub async fn get_top_songs(
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
    Query(params): Query<GetTopSongsParams>,
) -> Result<ApiResponse<TopSongsResponse>, AppError> {
    let count = params.count.unwrap_or(50).min(5000); // 默认50首，最多5000首
    let songs = state
        .browseing_service
        .get_top_songs(&params.artist, count)
        .await?;
    let song_responses: Vec<Song> = songs.into_iter().map(|dto| dto.into()).collect();

    let result = TopSongsResponse {
        top_songs: TopSongs {
            song: song_responses,
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getSongsByGenre - 获取指定流派的歌曲
pub async fn get_songs_by_genre(
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
    Query(params): Query<GetSongsByGenreParams>,
) -> Result<ApiResponse<SongsByGenreResponse>, AppError> {
    let count = params.count.unwrap_or(10).min(500); // 默认10首，最多500首
    let offset = params.offset.unwrap_or(0);

    let songs = state
        .browseing_service
        .get_songs_by_genre(&params.genre, count, offset)
        .await?;
    let song_responses: Vec<Song> = songs.into_iter().map(|dto| dto.into()).collect();

    let result = SongsByGenreResponse {
        songs_by_genre: SongsResponse {
            song: song_responses,
        },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// GET /rest/getGenres - 获取所有流派
pub async fn get_genres(
    Format(format): Format,
    axum::extract::State(state): axum::extract::State<BrowsingState>,
) -> Result<ApiResponse<GenresResponse>, AppError> {
    // 从歌曲和专辑中统计流派及其计数
    let genres = state.browseing_service.get_genres().await?;

    let genre_list: Vec<Genre> = genres
        .into_iter()
        .map(|(value, song_count, album_count)| Genre {
            value,
            song_count,
            album_count,
        })
        .collect();

    let result = GenresResponse {
        genres: Genres { genres: genre_list },
    };

    Ok(ApiResponse::ok(Some(result), format))
}

/// 组合状态，用于 browsing 路由
#[derive(Clone)]
pub struct BrowsingState {
    pub pool: Arc<sqlx::SqlitePool>,
    pub browseing_service: Arc<BrowsingService>,
}

pub fn routes(pool: Arc<sqlx::SqlitePool>, browseing_service: Arc<BrowsingService>) -> Router {
    let browsing_state = BrowsingState {
        pool: pool.clone(),
        browseing_service,
    };

    Router::new()
        .route("/rest/getIndexes", get(get_indexes))
        .route("/rest/getMusicDirectory", get(get_music_directory))
        .route("/rest/getGenres", get(get_genres))
        .route("/rest/getArtists", get(get_artists))
        .route("/rest/getArtist", get(get_artist))
        .route("/rest/getAlbum", get(get_album))
        .route("/rest/getSong", get(get_song))
        .route("/rest/getAlbumList", get(get_album_list2))
        .route("/rest/getAlbumList2", get(get_album_list2))
        .route("/rest/getRandomSongs", get(get_random_songs))
        .route("/rest/getArtistInfo", get(get_artist_info))
        .route("/rest/getArtistInfo2", get(get_artist_info2))
        .route("/rest/getTopSongs", get(get_top_songs))
        .route("/rest/getSongsByGenre", get(get_songs_by_genre))
        .with_state(browsing_state)
}
