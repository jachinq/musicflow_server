-- 播放列表表
CREATE TABLE playlists (
    id TEXT PRIMARY KEY,
    owner_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    comment TEXT,
    is_public INTEGER DEFAULT 0,
    song_count INTEGER DEFAULT 0,
    duration INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 索引
CREATE INDEX idx_playlists_owner_id ON playlists(owner_id);