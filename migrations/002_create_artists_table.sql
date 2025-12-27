-- 艺术家表
CREATE TABLE artists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    music_brainz_id TEXT,
    cover_art_path TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_artists_name ON artists(name);