#![allow(dead_code)]

use sqlx::SqlitePool;

use std::sync::Arc;
#[derive(Clone)]
pub struct CommState {
    pub pool: Arc<sqlx::SqlitePool>,
    pub song_service: Arc<SongService>,
}

pub struct SongService {
    pool: SqlitePool,
}

impl SongService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn detail_sql(&self) -> &str {
        "SELECT s.id, s.title, 
            ar.name as artist, 
            ar.id as artist_id, 
            al.name as album, 
            al.id as album_id, 
            s.track_number, s.disc_number, s.duration, s.bit_rate, s.genre,
            s.year, s.content_type, s.file_path as path,
            al.cover_art_path as cover_art
         FROM songs s
         JOIN albums al ON s.album_id = al.id
         JOIN artists ar ON s.artist_id = ar.id"
    }
}
