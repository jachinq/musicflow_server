//! 播放列表管理服务
//!
//! 负责处理播放列表相关的业务逻辑:
//! - 播放列表的创建、更新、删除
//! - 歌曲的添加、删除
//! - 权限检查 (所有者验证)
//! - 统计信息更新

use crate::error::AppError;
use crate::models::dto::{CreatePlaylistRequest, SongDto, UpdatePlaylistRequest, SongDetailDto};
use crate::services::ServiceContext;
use crate::utils::id_builder;
use futures::FutureExt;
use std::sync::Arc;

/// 播放列表基本信息
#[derive(Debug)]
pub struct PlaylistInfo {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub is_public: bool,
    pub song_count: i32,
    pub duration: i32,
}

/// 播放列表详细信息 (包含歌曲列表)
#[derive(Debug)]
pub struct PlaylistDetailInfo {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub is_public: bool,
    pub song_count: i32,
    pub duration: i32,
    pub songs: Vec<SongDetailDto>,
}

/// 播放列表管理服务
pub struct PlaylistService {
    ctx: Arc<ServiceContext>,
}

impl PlaylistService {
    /// 创建新的 PlaylistService
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// 检查播放列表所有者
    ///
    /// # 参数
    ///
    /// * `playlist_id` - 播放列表 ID
    /// * `user_id` - 用户 ID
    ///
    /// # 返回
    ///
    /// 如果用户是所有者返回 Ok,否则返回错误
    async fn check_playlist_owner(&self, playlist_id: &str, user_id: &str) -> Result<(), AppError> {
        let owner = sqlx::query_scalar::<_, String>("SELECT owner_id FROM playlists WHERE id = ?")
            .bind(playlist_id)
            .fetch_optional(&self.ctx.pool)
            .await?;

        match owner {
            Some(owner_id) if owner_id == user_id => Ok(()),
            Some(_) => Err(AppError::access_denied("Not playlist owner")),
            None => Err(AppError::not_found("Playlist")),
        }
    }

    /// 获取用户的所有播放列表
    ///
    /// # 参数
    ///
    /// * `user_id` - 用户 ID
    ///
    /// # 返回
    ///
    /// 返回用户自己的播放列表和公开的播放列表
    pub async fn get_playlists(&self, user_id: &str) -> Result<Vec<PlaylistInfo>, AppError> {
        let playlists = sqlx::query_as::<_, (String, String, String, bool, i32, i32)>(
            "SELECT id, name, owner_id, is_public, song_count, duration
             FROM playlists
             WHERE owner_id = ? OR is_public = true
             ORDER BY name",
        )
        .bind(user_id)
        .fetch_all(&self.ctx.pool)
        .await?;

        let results = playlists
            .into_iter()
            .map(
                |(id, name, owner_id, is_public, song_count, duration)| PlaylistInfo {
                    id,
                    name,
                    owner_id,
                    is_public,
                    song_count,
                    duration,
                },
            )
            .collect();

        Ok(results)
    }

    /// 获取播放列表详情
    ///
    /// # 参数
    ///
    /// * `playlist_id` - 播放列表 ID
    pub async fn get_playlist_detail(
        &self,
        playlist_id: &str,
    ) -> Result<PlaylistDetailInfo, AppError> {
        // 获取基本信息
        let (id, name, owner_id, is_public, song_count, duration) =
            sqlx::query_as::<_, (String, String, String, bool, i32, i32)>(
                "SELECT id, name, owner_id, is_public, song_count, duration
                 FROM playlists
                 WHERE id = ?",
            )
            .bind(playlist_id)
            .fetch_optional(&self.ctx.pool)
            .await?
            .ok_or_else(|| AppError::not_found("Playlist"))?;

        // 获取歌曲列表
        let songs = sqlx::query_as::<_, SongDetailDto>(
            "SELECT s.id, s.title, 
                    ar.name as artist, 
                    s.artist_id, 
                    al.name as album, 
                    s.album_id, 
                    s.track_number, s.disc_number, s.duration, s.bit_rate, s.genre,
                    s.year, s.content_type, s.file_path as path,
                    al.cover_art_path as cover_art, s.file_size, s.play_count
                FROM playlist_songs ps
                JOIN songs s ON ps.song_id = s.id
                JOIN albums al ON s.album_id = al.id
                JOIN artists ar ON s.artist_id = ar.id
                WHERE ps.playlist_id = ?
                ORDER BY ps.position",
        )
        .bind(playlist_id)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok(PlaylistDetailInfo {
            id,
            name,
            owner_id,
            is_public,
            song_count,
            duration,
            songs,
        })
    }

    /// 创建播放列表 (带事务保护)
    ///
    /// # 参数
    ///
    /// * `user_id` - 所有者 ID
    /// * `request` - 创建请求
    pub async fn create_playlist(
        &self,
        user_id: &str,
        request: CreatePlaylistRequest,
    ) -> Result<String, AppError> {
        let playlist_id = id_builder::generate_id();
        let user_id = user_id.to_string();
        let name = request.name.clone();
        let song_ids = request.song_id.clone();

        self.ctx
            .transaction(|tx| {
                async move {
                    // 创建播放列表
                    sqlx::query(
                        "INSERT INTO playlists (id, owner_id, name, comment, is_public, song_count, duration, created_at, updated_at)
                         VALUES (?, ?, ?, ?, ?, 0, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    )
                    .bind(&playlist_id)
                    .bind(&user_id)
                    .bind(&name)
                    .bind(None::<String>)
                    .bind(false)
                    .execute(&mut **tx)
                    .await?;

                    // 如果有初始歌曲,添加到播放列表
                    if let Some(song_ids) = song_ids {
                        for (position, song_id) in song_ids.iter().enumerate() {
                            sqlx::query(
                                "INSERT INTO playlist_songs (playlist_id, song_id, position)
                                 VALUES (?, ?, ?)",
                            )
                            .bind(&playlist_id)
                            .bind(song_id)
                            .bind(position as i32)
                            .execute(&mut **tx)
                            .await?;
                        }

                        // 更新统计
                        Self::update_stats_in_tx(tx, &playlist_id).await?;
                    }

                    Ok(playlist_id.clone())
                }
                .boxed()
            })
            .await
    }

    /// 更新播放列表 (带事务保护)
    ///
    /// # 参数
    ///
    /// * `playlist_id` - 播放列表 ID
    /// * `user_id` - 用户 ID (所有者验证)
    /// * `request` - 更新请求
    ///
    /// # 事务保护
    ///
    /// 包含多个操作:
    /// 1. 更新基本信息
    /// 2. 添加歌曲
    /// 3. 删除歌曲并重排序
    /// 4. 更新统计信息
    pub async fn update_playlist(
        &self,
        user_id: &str,
        request: UpdatePlaylistRequest,
    ) -> Result<(), AppError> {
        // 权限检查
        self.check_playlist_owner(&request.playlist_id, user_id)
            .await?;

        let playlist_id = request.playlist_id.to_string();
        let name = request.name.clone();
        let comment = request.comment.clone();
        let public = request.public;
        let song_ids_to_add = request.song_id_to_add.clone();
        let indices_to_remove = request.song_index_to_remove.clone();

        self.ctx
            .transaction(|tx| {
                async move {
                    // 1. 更新基本信息
                    if name.is_some() || comment.is_some() || public.is_some() {
                        let mut query_parts = Vec::new();

                        if name.is_some() {
                            query_parts.push("name = ?");
                        }
                        if comment.is_some() {
                            query_parts.push("comment = ?");
                        }
                        if public.is_some() {
                            query_parts.push("is_public = ?");
                        }

                        let query_sql = format!(
                            "UPDATE playlists SET {}, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                            query_parts.join(", ")
                        );

                        let mut query = sqlx::query(&query_sql);

                        if let Some(n) = name {
                            query = query.bind(n);
                        }
                        if let Some(c) = comment {
                            query = query.bind(c);
                        }
                        if let Some(p) = public {
                            query = query.bind(p);
                        }

                        query.bind(&playlist_id).execute(&mut **tx).await?;
                    }

                    // 2. 添加歌曲
                    if let Some(song_ids) = song_ids_to_add {
                        let current_count = sqlx::query_scalar::<_, i32>(
                            "SELECT COUNT(*) FROM playlist_songs WHERE playlist_id = ?",
                        )
                        .bind(&playlist_id)
                        .fetch_one(&mut **tx)
                        .await?;

                        for (offset, song_id) in song_ids.iter().enumerate() {
                            sqlx::query(
                                "INSERT INTO playlist_songs (playlist_id, song_id, position)
                                 VALUES (?, ?, ?)",
                            )
                            .bind(&playlist_id)
                            .bind(song_id)
                            .bind(current_count + offset as i32)
                            .execute(&mut **tx)
                            .await?;
                        }
                    }

                    // 3. 删除歌曲并重排序
                    if let Some(indices) = indices_to_remove {
                        for index in indices {
                            sqlx::query(
                                "DELETE FROM playlist_songs WHERE playlist_id = ? AND position = ?",
                            )
                            .bind(&playlist_id)
                            .bind(index)
                            .execute(&mut **tx)
                            .await?;
                        }

                        // 重新排序位置
                        sqlx::query(
                            "UPDATE playlist_songs
                             SET position = (
                                 SELECT COUNT(*) FROM playlist_songs ps2
                                 WHERE ps2.playlist_id = playlist_songs.playlist_id
                                 AND ps2.position < playlist_songs.position
                             )
                             WHERE playlist_id = ?",
                        )
                        .bind(&playlist_id)
                        .execute(&mut **tx)
                        .await?;
                    }

                    // 4. 更新统计
                    Self::update_stats_in_tx(tx, &playlist_id).await?;

                    Ok(())
                }
                .boxed()
            })
            .await
    }

    /// 删除播放列表
    ///
    /// # 参数
    ///
    /// * `playlist_id` - 播放列表 ID
    /// * `user_id` - 用户 ID (所有者验证)
    pub async fn delete_playlist(&self, playlist_id: &str, user_id: &str) -> Result<(), AppError> {
        // 权限检查
        self.check_playlist_owner(playlist_id, user_id).await?;

        // 删除播放列表 (级联删除歌曲关联)
        sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(playlist_id)
            .execute(&self.ctx.pool)
            .await?;

        Ok(())
    }

    /// 追加歌曲到播放列表 (带事务保护)
    ///
    /// # 参数
    ///
    /// * `playlist_id` - 播放列表 ID
    /// * `user_id` - 用户 ID (所有者验证)
    /// * `song_ids` - 要添加的歌曲 ID 列表
    pub async fn append_songs(
        &self,
        playlist_id: &str,
        user_id: &str,
        song_ids: Vec<String>,
    ) -> Result<(), AppError> {
        // 权限检查
        self.check_playlist_owner(playlist_id, user_id).await?;

        let playlist_id = playlist_id.to_string();

        self.ctx
            .transaction(|tx| {
                async move {
                    // 获取当前歌曲数量
                    let current_count = sqlx::query_scalar::<_, i32>(
                        "SELECT COUNT(*) FROM playlist_songs WHERE playlist_id = ?",
                    )
                    .bind(&playlist_id)
                    .fetch_one(&mut **tx)
                    .await?;

                    // 添加歌曲
                    for (offset, song_id) in song_ids.iter().enumerate() {
                        sqlx::query(
                            "INSERT INTO playlist_songs (playlist_id, song_id, position)
                             VALUES (?, ?, ?)",
                        )
                        .bind(&playlist_id)
                        .bind(song_id)
                        .bind(current_count + offset as i32)
                        .execute(&mut **tx)
                        .await?;
                    }

                    // 更新统计
                    Self::update_stats_in_tx(tx, &playlist_id).await?;

                    Ok(())
                }
                .boxed()
            })
            .await
    }

    /// 在事务中更新播放列表统计信息 (私有方法)
    ///
    /// # 参数
    ///
    /// * `tx` - 事务连接
    /// * `playlist_id` - 播放列表 ID
    async fn update_stats_in_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        playlist_id: &str,
    ) -> Result<(), AppError> {
        // 计算歌曲数量和总时长
        let stats = sqlx::query_as::<_, (i32, Option<i32>)>(
            "SELECT
                COUNT(*) as count,
                SUM(duration) as total_duration
             FROM playlist_songs ps
             JOIN songs s ON ps.song_id = s.id
             WHERE ps.playlist_id = ?",
        )
        .bind(playlist_id)
        .fetch_one(&mut **tx)
        .await?;

        let (song_count, total_duration) = stats;
        let duration = total_duration.unwrap_or(0);

        // 更新播放列表
        sqlx::query(
            "UPDATE playlists
             SET song_count = ?, duration = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(song_count)
        .bind(duration)
        .bind(playlist_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // 创建测试表
        sqlx::query(
            "CREATE TABLE playlists (
                id TEXT PRIMARY KEY,
                owner_id TEXT NOT NULL,
                name TEXT NOT NULL,
                comment TEXT,
                is_public BOOLEAN DEFAULT 0,
                song_count INTEGER DEFAULT 0,
                duration INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE playlist_songs (
                playlist_id TEXT NOT NULL,
                song_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                PRIMARY KEY (playlist_id, song_id)
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE songs (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                duration INTEGER DEFAULT 0,
                artist_id TEXT,
                album_id TEXT
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE artists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE albums (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        // 插入测试数据
        sqlx::query("INSERT INTO artists (id, name) VALUES ('artist1', 'Test Artist')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO albums (id, name) VALUES ('album1', 'Test Album')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO songs (id, title, duration, artist_id, album_id) VALUES ('song1', 'Song 1', 180, 'artist1', 'album1')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO songs (id, title, duration, artist_id, album_id) VALUES ('song2', 'Song 2', 200, 'artist1', 'album1')")
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    fn create_service(pool: SqlitePool) -> PlaylistService {
        let ctx = Arc::new(ServiceContext::new(pool));
        PlaylistService::new(ctx)
    }

    #[tokio::test]
    async fn test_create_playlist_empty() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        let request = CreatePlaylistRequest {
            name: "Test Playlist".to_string(),
            song_id: None,
        };

        let result = service.create_playlist("user1", request).await;
        assert!(result.is_ok());

        let playlist_id = result.unwrap();

        // 验证播放列表已创建
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM playlists WHERE id = ?")
            .bind(&playlist_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_create_playlist_with_songs() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        let request = CreatePlaylistRequest {
            name: "Test Playlist".to_string(),
            song_id: Some(vec!["song1".to_string(), "song2".to_string()]),
        };

        let result = service.create_playlist("user1", request).await;
        assert!(result.is_ok());

        let playlist_id = result.unwrap();

        // 验证歌曲已添加
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM playlist_songs WHERE playlist_id = ?")
                .bind(&playlist_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 2);

        // 验证统计已更新
        let (song_count, duration): (i32, i32) =
            sqlx::query_as("SELECT song_count, duration FROM playlists WHERE id = ?")
                .bind(&playlist_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(song_count, 2);
        assert_eq!(duration, 380); // 180 + 200
    }

    #[tokio::test]
    async fn test_check_playlist_owner() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        // 创建播放列表
        let request = CreatePlaylistRequest {
            name: "Test Playlist".to_string(),
            song_id: None,
        };
        let playlist_id = service.create_playlist("user1", request).await.unwrap();

        // 所有者应该通过检查
        let result = service.check_playlist_owner(&playlist_id, "user1").await;
        assert!(result.is_ok());

        // 非所有者应该失败
        let result = service.check_playlist_owner(&playlist_id, "user2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_playlist_transaction() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        // 创建播放列表
        let request = CreatePlaylistRequest {
            name: "Test Playlist".to_string(),
            song_id: None,
        };
        let playlist_id = service.create_playlist("user1", request).await.unwrap();

        // 更新播放列表
        let update_request = UpdatePlaylistRequest {
            playlist_id: playlist_id.clone(),
            name: Some("Updated Playlist".to_string()),
            comment: Some("Test comment".to_string()),
            public: Some(true),
            song_id_to_add: Some(vec!["song1".to_string()]),
            song_index_to_remove: None,
        };

        let result = service.update_playlist("user1", update_request).await;
        assert!(result.is_ok());

        // 验证更新
        let (name, comment, public): (String, Option<String>, bool) =
            sqlx::query_as("SELECT name, comment, is_public FROM playlists WHERE id = ?")
                .bind(&playlist_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(name, "Updated Playlist");
        assert_eq!(comment, Some("Test comment".to_string()));
        assert!(public);

        // 验证歌曲已添加
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM playlist_songs WHERE playlist_id = ?")
                .bind(&playlist_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_delete_playlist() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        // 创建播放列表
        let request = CreatePlaylistRequest {
            name: "Test Playlist".to_string(),
            song_id: None,
        };
        let playlist_id = service.create_playlist("user1", request).await.unwrap();

        // 删除播放列表
        let result = service.delete_playlist(&playlist_id, "user1").await;
        assert!(result.is_ok());

        // 验证已删除
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM playlists WHERE id = ?")
            .bind(&playlist_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_append_songs() {
        let pool = setup_test_db().await;
        let service = create_service(pool.clone());

        // 创建播放列表
        let request = CreatePlaylistRequest {
            name: "Test Playlist".to_string(),
            song_id: Some(vec!["song1".to_string()]),
        };
        let playlist_id = service.create_playlist("user1", request).await.unwrap();

        // 追加歌曲
        let result = service
            .append_songs(&playlist_id, "user1", vec!["song2".to_string()])
            .await;
        assert!(result.is_ok());

        // 验证歌曲已添加
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM playlist_songs WHERE playlist_id = ?")
                .bind(&playlist_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 2);

        // 验证统计已更新
        let (song_count, duration): (i32, i32) =
            sqlx::query_as("SELECT song_count, duration FROM playlists WHERE id = ?")
                .bind(&playlist_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(song_count, 2);
        assert_eq!(duration, 380);
    }
}
