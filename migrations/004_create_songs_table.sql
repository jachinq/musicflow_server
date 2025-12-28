-- 歌曲表
CREATE TABLE songs (
    id TEXT PRIMARY KEY,
    album_id TEXT REFERENCES albums(id) ON DELETE CASCADE,
    artist_id TEXT REFERENCES artists(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    track_number INTEGER,
    disc_number INTEGER,
    duration INTEGER NOT NULL,
    bit_rate INTEGER,
    genre TEXT,
    year INTEGER,
    content_type TEXT,
    file_path TEXT NOT NULL,
    file_size BIGINT,
    lyrics TEXT,
    play_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 索引
CREATE INDEX idx_songs_album_id ON songs(album_id);
CREATE INDEX idx_songs_artist_id ON songs(artist_id);
CREATE INDEX idx_songs_title ON songs(title);