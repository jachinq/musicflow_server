-- 专辑表
CREATE TABLE albums (
    id TEXT PRIMARY KEY,
    artist_id TEXT REFERENCES artists(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    year INTEGER,
    genre TEXT,
    cover_art_path TEXT,
    path TEXT NOT NULL,
    song_count INTEGER DEFAULT 0,
    duration INTEGER DEFAULT 0,
    play_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 索引
CREATE INDEX idx_albums_artist_id ON albums(artist_id);
CREATE INDEX idx_albums_name ON albums(name);