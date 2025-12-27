-- 用户表
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    api_password TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    is_admin INTEGER DEFAULT 0,
    max_bitrate INTEGER DEFAULT 320,
    download_role INTEGER DEFAULT 1,
    upload_role INTEGER DEFAULT 0,
    playlist_role INTEGER DEFAULT 1,
    cover_art_role INTEGER DEFAULT 1,
    comment_role INTEGER DEFAULT 0,
    podcast_role INTEGER DEFAULT 0,
    share_role INTEGER DEFAULT 1,
    video_conversion_role INTEGER DEFAULT 0,
    scrobbling_enabled INTEGER DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);