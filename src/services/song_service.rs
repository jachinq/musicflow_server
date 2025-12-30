#![allow(dead_code)]

use sqlx::SqlitePool;

use std::sync::Arc;
#[derive(Clone)]
pub struct CommState {
    pub pool: Arc<sqlx::SqlitePool>,
}

pub struct SongService {
    pool: SqlitePool,
}

impl SongService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
