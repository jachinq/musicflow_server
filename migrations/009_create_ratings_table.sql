-- 评分表
CREATE TABLE ratings (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    artist_id TEXT REFERENCES artists(id) ON DELETE CASCADE,
    album_id TEXT REFERENCES albums(id) ON DELETE CASCADE,
    song_id TEXT REFERENCES songs(id) ON DELETE CASCADE,
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, artist_id),
    UNIQUE(user_id, album_id),
    UNIQUE(user_id, song_id)
);

-- 索引
CREATE INDEX idx_ratings_user_artist ON ratings(user_id, artist_id);
CREATE INDEX idx_ratings_user_album ON ratings(user_id, album_id);
CREATE INDEX idx_ratings_user_song ON ratings(user_id, song_id);