-- 播放列表歌曲关联表
CREATE TABLE playlist_songs (
    playlist_id TEXT REFERENCES playlists(id) ON DELETE CASCADE,
    song_id TEXT REFERENCES songs(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (playlist_id, song_id)
);

-- 索引
CREATE INDEX idx_playlist_songs_playlist ON playlist_songs(playlist_id);
CREATE INDEX idx_playlist_songs_song ON playlist_songs(song_id);