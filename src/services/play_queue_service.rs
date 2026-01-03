//! 播放队列管理服务
//!
//! 负责处理播放队列相关的业务逻辑:
//! - 获取用户的播放队列状态
//! - 保存播放队列状态（包括歌曲列表、当前播放位置）
//! - 每个用户只有一个播放队列

use crate::error::AppError;
use crate::models::dto::SongDetailDto;
use crate::models::entities::{PlayQueue, PlayQueueSong};
use crate::services::ServiceContext;
use crate::utils::{id_builder, sql_utils};
use chrono::Utc;
use futures::FutureExt;
use std::sync::Arc;

/// 播放队列详细信息（包含歌曲列表）
#[derive(Debug)]
pub struct PlayQueueDetail {
    pub current_song_id: Option<String>,
    pub position: i64,
    pub changed_at: String,
    pub changed_by: String,
    pub username: String,
    pub songs: Vec<SongDetailDto>,
}

/// 播放队列管理服务
pub struct PlayQueueService {
    ctx: Arc<ServiceContext>,
}

impl PlayQueueService {
    /// 创建新的 PlayQueueService
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// 获取用户的播放队列
    pub async fn get_play_queue(
        &self,
        user_id: &str,
    ) -> Result<Option<PlayQueueDetail>, AppError> {
        // 查询播放队列主记录
        let queue = sqlx::query_as::<_, PlayQueue>(
            "SELECT * FROM play_queue WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.ctx.pool)
        .await?;

        let queue = match queue {
            Some(q) => q,
            None => return Ok(None),
        };

        // 查询用户名
        let username: String = sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&self.ctx.pool)
            .await?;

        // 查询播放队列中的歌曲（按顺序）
        let songs = sqlx::query_as::<_, SongDetailDto>(
            &format!(r#"{}
            JOIN play_queue_songs as pqs ON pqs.song_id = s.id
            WHERE pqs.play_queue_id = ?
            ORDER BY pqs.song_order ASC
            "#, sql_utils::detail_sql()),
        )
        .bind(&queue.id)
        .fetch_all(&self.ctx.pool)
        .await?;

        Ok(Some(PlayQueueDetail {
            current_song_id: queue.current_song_id,
            position: queue.position,
            changed_at: queue.changed_at.to_rfc3339(),
            changed_by: queue.changed_by,
            username,
            songs,
        }))
    }

    /// 保存用户的播放队列
    pub async fn save_play_queue(
        &self,
        user_id: &str,
        song_ids: Vec<String>,
        current: Option<String>,
        position: Option<i64>,
        changed_by: &str,
    ) -> Result<(), AppError> {
        let position = position.unwrap_or(0);
        let now = Utc::now();

        // 克隆字符串以避免生命周期问题
        let user_id = user_id.to_string();
        let changed_by = changed_by.to_string();

        self.ctx
            .transaction(|tx| {
                async move {
                    // 检查是否已存在播放队列
                    let existing_queue: Option<String> =
                        sqlx::query_scalar("SELECT id FROM play_queue WHERE user_id = ?")
                            .bind(&user_id)
                            .fetch_optional(&mut **tx)
                            .await?;

                    let queue_id = if let Some(id) = existing_queue {
                        // 更新现有播放队列
                        sqlx::query(
                            r#"
                            UPDATE play_queue
                            SET current_song_id = ?, position = ?, changed_at = ?, changed_by = ?, updated_at = ?
                            WHERE id = ?
                            "#,
                        )
                        .bind(&current)
                        .bind(position)
                        .bind(now)
                        .bind(&changed_by)
                        .bind(now)
                        .bind(&id)
                        .execute(&mut **tx)
                        .await?;

                        // 删除旧的歌曲关联
                        sqlx::query("DELETE FROM play_queue_songs WHERE play_queue_id = ?")
                            .bind(&id)
                            .execute(&mut **tx)
                            .await?;

                        id
                    } else {
                        // 创建新的播放队列
                        let new_id = id_builder::generate_id();
                        sqlx::query(
                            r#"
                            INSERT INTO play_queue (id, user_id, current_song_id, position, changed_at, changed_by, updated_at)
                            VALUES (?, ?, ?, ?, ?, ?, ?)
                            "#,
                        )
                        .bind(&new_id)
                        .bind(&user_id)
                        .bind(&current)
                        .bind(position)
                        .bind(now)
                        .bind(&changed_by)
                        .bind(now)
                        .execute(&mut **tx)
                        .await?;

                        new_id
                    };

                    // 批量插入歌曲关联
                    for (index, song_id) in song_ids.iter().enumerate() {
                        let pqs = PlayQueueSong::new(
                            queue_id.clone(),
                            song_id.clone(),
                            index as i32,
                        );

                        sqlx::query(
                            r#"
                            INSERT INTO play_queue_songs (id, play_queue_id, song_id, song_order)
                            VALUES (?, ?, ?, ?)
                            "#,
                        )
                        .bind(&pqs.id)
                        .bind(&pqs.play_queue_id)
                        .bind(&pqs.song_id)
                        .bind(&pqs.song_order)
                        .execute(&mut **tx)
                        .await?;
                    }

                    Ok(())
                }
                .boxed()
            })
            .await?;

        Ok(())
    }
}
