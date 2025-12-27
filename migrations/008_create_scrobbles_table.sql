-- 播放记录表
CREATE TABLE scrobbles (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    song_id TEXT REFERENCES songs(id) ON DELETE CASCADE,
    timestamp TEXT NOT NULL,
    submission INTEGER DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 索引
CREATE INDEX idx_scrobbles_user_timestamp ON scrobbles(user_id, timestamp);
CREATE INDEX idx_scrobbles_song ON scrobbles(song_id);