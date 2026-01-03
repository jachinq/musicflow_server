-- 创建播放队列表
CREATE TABLE play_queue (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL UNIQUE,  -- 每个用户只有一个播放队列
    current_song_id TEXT,           -- 当前播放的歌曲 ID
    position INTEGER DEFAULT 0,     -- 当前歌曲播放位置（毫秒）
    changed_at TEXT NOT NULL,       -- 最后修改时间（ISO 8601）
    changed_by TEXT NOT NULL,       -- 修改来源客户端
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (current_song_id) REFERENCES songs(id) ON DELETE SET NULL
);

-- 创建播放队列歌曲关联表
CREATE TABLE play_queue_songs (
    id TEXT PRIMARY KEY,
    play_queue_id TEXT NOT NULL,
    song_id TEXT NOT NULL,
    song_order INTEGER NOT NULL,    -- 歌曲在队列中的顺序
    FOREIGN KEY (play_queue_id) REFERENCES play_queue(id) ON DELETE CASCADE,
    FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE,
    UNIQUE(play_queue_id, song_order)
);

-- 创建索引优化查询性能
CREATE INDEX idx_play_queue_user ON play_queue(user_id);
CREATE INDEX idx_play_queue_songs_queue ON play_queue_songs(play_queue_id);
CREATE INDEX idx_play_queue_songs_order ON play_queue_songs(play_queue_id, song_order);
