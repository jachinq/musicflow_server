-- 收藏表
CREATE TABLE starred (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    artist_id TEXT REFERENCES artists(id) ON DELETE CASCADE,
    album_id TEXT REFERENCES albums(id) ON DELETE CASCADE,
    song_id TEXT REFERENCES songs(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, artist_id),
    UNIQUE(user_id, album_id),
    UNIQUE(user_id, song_id)
);

-- 索引
CREATE INDEX idx_starred_user_artist ON starred(user_id, artist_id);
CREATE INDEX idx_starred_user_album ON starred(user_id, album_id);
CREATE INDEX idx_starred_user_song ON starred(user_id, song_id);