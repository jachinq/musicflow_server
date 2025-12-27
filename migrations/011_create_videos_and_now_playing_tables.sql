-- 视频表
CREATE TABLE IF NOT EXISTS videos (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL,
    content_type TEXT NOT NULL,
    file_size BIGINT,
    duration INTEGER,
    bit_rate INTEGER,
    width INTEGER,
    height INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_videos_title ON videos(title);

-- 正在播放表（用于记录当前正在播放的歌曲）
CREATE TABLE IF NOT EXISTS now_playing (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    username TEXT NOT NULL,
    song_id TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    started_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_now_playing_user ON now_playing(user_id);
CREATE INDEX idx_now_playing_started_at ON now_playing(started_at DESC);
