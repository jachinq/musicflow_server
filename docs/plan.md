# Music Flow Server 项目计划

## 项目概述
基于 Subsonic API 1.16.1 的音乐流媒体服务器，使用 Rust + SQLite 构建。

## 阶段1: 项目初始化 ✅
- [x] 创建 Rust 项目结构
- [x] 配置 Cargo.toml 依赖
- [x] 设置基础目录结构
- [x] 配置环境变量 (.env)

## 阶段2: 数据库设计 ✅
- [x] 创建数据库迁移文件 (9个迁移文件)
- [x] 实现数据库连接模块
- [x] 实现配置管理模块
- [x] 验证数据库迁移正常工作
- [x] 修复依赖缺失问题

### 数据库表结构
1. **users** - 用户表
2. **artists** - 艺术家表
3. **albums** - 专辑表
4. **songs** - 歌曲表
5. **playlists** - 播放列表表
6. **playlist_songs** - 播放列表歌曲关联表
7. **starred** - 收藏表
8. **scrobbles** - 播放记录表
9. **ratings** - 评分表

### 数据模型
- **models/** - 数据库实体和 API 响应模型
  - user.rs - 用户模型
  - artist.rs - 艺术家模型
  - album.rs - 专辑模型
  - song.rs - 歌曲模型
  - playlist.rs - 播放列表模型
  - response.rs - Subsonic API 响应模型
  - starred.rs - 收藏模型
  - scrobble.rs - 播放记录模型
  - rating.rs - 评分模型

### 核心模块
- **database/** - 数据库连接和迁移管理
- **config/** - 应用配置管理
- **error/** - 错误处理系统
- **utils/** - 工具函数 (认证、哈希等)
- **middleware/** - 中间件 (认证等)
- **handlers/** - API 请求处理器

## 阶段3: 核心功能实现 ✅
- [x] 用户认证系统
- [x] Subsonic API 端点实现
- [x] 音乐库扫描和索引
- [x] 流媒体服务
- [x] 搜索功能

## 阶段4: 高级功能 (待开发)
- [ ] 播放列表管理
- [ ] 收藏和评分
- [ ] 播放记录 (Scrobble)
- [ ] 用户权限管理
- [ ] 实时搜索和过滤

## 阶段5: 测试和优化 (待开发)
- [ ] 单元测试
- [ ] 集成测试
- [ ] 性能优化
- [ ] 错误处理完善

## 当前状态
✅ **阶段3 已完成**

项目核心功能已实现，所有代码可以正常编译。

### 已完成
- ✅ 阶段1: 项目初始化
- ✅ 阶段2: 数据库设计（9个迁移文件）
- ✅ 阶段3: 核心功能实现
  - ✅ 用户认证系统（AuthService, JWT + Subsonic认证）
  - ✅ 认证中间件
  - ✅ 8个API端点模块（system, auth, browsing, search, stream, playlist, user, library）
  - ✅ 音乐库扫描服务（ScanService, 支持多种音频格式）
  - ✅ 流媒体服务
  - ✅ 搜索功能
  - ✅ 播放列表管理
  - ✅ 用户管理
  - ✅ 库管理（收藏/评分/Scrobble）

### 验证结果
- ✅ 项目编译成功（0个错误，仅有少量警告）
- ✅ 所有数据模型定义完成
- ✅ 主应用框架完整
- ✅ 路由配置完成
- ✅ 中间件系统就绪

### 下一步
阶段3已完成，可以进入阶段4 - 高级功能开发。

---

## 原始计划详情

## 技术栈选择

### Web 框架
- **Axum**: 高性能、类型安全的异步 Web 框架，适合 REST API
- **Tokio**: 异步运行时
- **Tower**: 中间件层

### 数据库
- **SQLx**: 异步 SQL 数据库工具，支持 PostgreSQL/MySQL/SQLite
- **推荐**: SQLite（开发/小型部署，无需额外数据库服务）

### 音频处理
- **symphonia**: 音频文件元数据提取和解码
- **hound**: WAV 文件处理（可选）

### 其他依赖
- **serde**: JSON 序列化/反序列化
- **serde_json**: JSON 处理
- **quick-xml**: XML 响应生成（Subsonic 默认格式）
- **chrono**: 时间处理
- **uuid**: UUID 生成
- **bcrypt**: 密码哈希
- **jsonwebtoken**: JWT 令牌生成（用于认证）
- **tracing**: 日志记录
- **dotenvy**: 环境变量管理

## 项目架构

```
musicflow_server/
├── src/
│   ├── main.rs              # 应用入口
│   ├── lib.rs               # 模块声明
│   ├── config/              # 配置管理
│   │   ├── mod.rs
│   │   └── app_config.rs
│   ├── models/              # 数据模型
│   │   ├── mod.rs
│   │   ├── user.rs          # 用户模型
│   │   ├── artist.rs        # 艺术家模型
│   │   ├── album.rs         # 专辑模型
│   │   ├── song.rs          # 歌曲模型
│   │   ├── playlist.rs      # 播放列表模型
│   │   └── response.rs      # API 响应模型
│   ├── schema/              # 数据库模式
│   │   ├── mod.rs
│   │   └── schema.sql
│   ├── handlers/            # API 处理器
│   │   ├── mod.rs
│   │   ├── auth.rs          # 认证相关
│   │   ├── browsing.rs      # 浏览类端点
│   │   ├── search.rs        # 搜索类端点
│   │   ├── streaming.rs     # 流媒体端点
│   │   ├── playlist.rs      # 播放列表端点
│   │   ├── media.rs         # 媒体检索端点
│   │   ├── library.rs       # 库管理端点
│   │   ├── chat.rs          # 聊天端点
│   │   ├── user.rs          # 用户管理端点
│   │   └── system.rs        # 系统端点
│   ├── services/            # 业务逻辑
│   │   ├── mod.rs
│   │   ├── auth_service.rs  # 认证服务
│   │   ├── music_service.rs # 音乐库服务
│   │   ├── scan_service.rs  # 扫描服务
│   │   └── scrobble_service.rs # 播放记录服务
│   ├── database/            # 数据库操作
│   │   ├── mod.rs
│   │   └── connection.rs
│   ├── middleware/          # 中间件
│   │   ├── mod.rs
│   │   ├── auth_middleware.rs
│   │   └── rate_limit.rs
│   ├── utils/               # 工具函数
│   │   ├── mod.rs
│   │   ├── auth_utils.rs    # 认证工具
│   │   ├── xml_utils.rs     # XML 响应工具
│   │   ├── file_utils.rs    # 文件处理工具
│   │   └── hash_utils.rs    # 哈希工具
│   └── error.rs             # 错误处理
├── migrations/              # 数据库迁移
│   ├── 001_create_users_table.sql
│   ├── 002_create_artists_table.sql
│   ├── 003_create_albums_table.sql
│   ├── 004_create_songs_table.sql
│   ├── 005_create_playlists_table.sql
│   ├── 006_create_scrobble_table.sql
│   └── 007_create_starred_table.sql
├── tests/                   # 测试
│   ├── integration/
│   └── unit/
├── .env                     # 环境变量
├── Cargo.toml
└── README.md
```

## 实现步骤

### 阶段 1: 项目基础搭建 (Day 1)

#### 1.1 更新 Cargo.toml
```toml
[package]
name = "musicflow_server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web 框架
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors"] }

# 数据库
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls", "migrate", "chrono", "uuid"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
quick-xml = { version = "0.31", features = ["serialize"] }

# 音频处理
symphonia = { version = "0.5", features = ["all"] }

# 工具库
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
bcrypt = "0.15"
jsonwebtoken = "9.2"
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 异步
futures = "0.3"
```

#### 1.2 创建环境配置
```bash
# .env
DATABASE_URL=sqlite:music_flow.db
PORT=4040
HOST=127.0.0.1
JWT_SECRET=your-secret-key-change-in-production
MUSIC_LIBRARY_PATH=/path/to/your/music
```

### 阶段 2: 数据库设计 (Day 1-2)

#### 2.1 核心数据表设计

```sql
-- 用户表
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    is_admin BOOLEAN DEFAULT FALSE,
    max_bitrate INTEGER DEFAULT 320,
    download_role BOOLEAN DEFAULT TRUE,
    upload_role BOOLEAN DEFAULT FALSE,
    playlist_role BOOLEAN DEFAULT TRUE,
    cover_art_role BOOLEAN DEFAULT TRUE,
    comment_role BOOLEAN DEFAULT FALSE,
    podcast_role BOOLEAN DEFAULT FALSE,
    share_role BOOLEAN DEFAULT TRUE,
    video_conversion_role BOOLEAN DEFAULT FALSE,
    scrobbling_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 艺术家表
CREATE TABLE artists (
    id TEXT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    music_brainz_id VARCHAR(255),
    cover_art_path VARCHAR(500),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 专辑表
CREATE TABLE albums (
    id TEXT PRIMARY KEY,
    artist_id TEXT REFERENCES artists(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    year INTEGER,
    genre VARCHAR(100),
    cover_art_path VARCHAR(500),
    path VARCHAR(500) NOT NULL,
    song_count INTEGER DEFAULT 0,
    duration INTEGER DEFAULT 0,
    play_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 歌曲表
CREATE TABLE songs (
    id TEXT PRIMARY KEY,
    album_id TEXT REFERENCES albums(id) ON DELETE CASCADE,
    artist_id TEXT REFERENCES artists(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    track_number INTEGER,
    disc_number INTEGER,
    duration INTEGER NOT NULL,
    bit_rate INTEGER,
    genre VARCHAR(100),
    year INTEGER,
    content_type VARCHAR(50),
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT,
    play_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 播放列表表
CREATE TABLE playlists (
    id TEXT PRIMARY KEY,
    owner_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    comment TEXT,
    is_public BOOLEAN DEFAULT FALSE,
    song_count INTEGER DEFAULT 0,
    duration INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 播放列表歌曲关联表
CREATE TABLE playlist_songs (
    playlist_id TEXT REFERENCES playlists(id) ON DELETE CASCADE,
    song_id TEXT REFERENCES songs(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (playlist_id, song_id)
);

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

-- 播放记录表
CREATE TABLE scrobbles (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    song_id TEXT REFERENCES songs(id) ON DELETE CASCADE,
    timestamp TIMESTAMP NOT NULL,
    submission BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

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

-- 索引优化
CREATE INDEX idx_songs_album_id ON songs(album_id);
CREATE INDEX idx_songs_artist_id ON songs(artist_id);
CREATE INDEX idx_albums_artist_id ON albums(artist_id);
CREATE INDEX idx_scrobbles_user_timestamp ON scrobbles(user_id, timestamp);
CREATE INDEX idx_starred_user_artist ON starred(user_id, artist_id);
CREATE INDEX idx_starred_user_album ON starred(user_id, album_id);
CREATE INDEX idx_starred_user_song ON starred(user_id, song_id);
```

### 阶段 3: 核心数据模型 (Day 2-3)

#### 3.1 用户认证模型
```rust
// models/user.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: String,
    pub is_admin: bool,
    pub max_bitrate: i32,
    pub download_role: bool,
    pub upload_role: bool,
    pub playlist_role: bool,
    pub cover_art_role: bool,
    pub comment_role: bool,
    pub podcast_role: bool,
    pub share_role: bool,
    pub video_conversion_role: bool,
    pub scrobbling_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    pub is_admin: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub username: String,
    pub email: String,
    pub admin: bool,
    pub scrobbling_enabled: bool,
    pub max_bit_rate: i32,
    pub download_role: bool,
    pub upload_role: bool,
    pub playlist_role: bool,
    pub cover_art_role: bool,
    pub comment_role: bool,
    pub podcast_role: bool,
    pub share_role: bool,
    pub video_conversion_role: bool,
}
```

#### 3.2 Subsonic 响应模型
```rust
// models/response.rs
use serde::{Deserialize, Serialize};
use quick_xml::se::Serializer;
use std::fmt;

// Subsonic 响应容器
#[derive(Debug, Serialize, Deserialize)]
pub struct SubsonicResponse<T> {
    #[serde(rename = "subsonic-response")]
    pub response: ResponseContainer<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseContainer<T> {
    pub status: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SubsonicError>,
    #[serde(flatten)]
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubsonicError {
    pub code: i32,
    pub message: String,
}

// 艺术家索引
#[derive(Debug, Serialize, Deserialize)]
pub struct Indexes {
    pub last_modified: i64,
    #[serde(rename = "index")]
    pub indexes: Vec<Index>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub artist: Vec<Artist>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_count: Option<i32>,
}

// 歌曲详情
#[derive(Debug, Serialize, Deserialize)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    pub duration: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<i32>,
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}
```

### 阶段 4: 认证系统 (Day 3-4)

#### 4.1 认证工具
```rust
// utils/auth_utils.rs
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, Duration};
use rand::Rng;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub exp: usize,
    pub iat: usize,
}

// 生成密码哈希
pub fn hash_password(password: &str) -> Result<String, anyhow::Error> {
    hash(password, DEFAULT_COST).map_err(Into::into)
}

// 验证密码
pub fn verify_password(password: &str, hash: &str) -> Result<bool, anyhow::Error> {
    verify(password, hash).map_err(Into::into)
}

// 生成 JWT 令牌
pub fn generate_token(user_id: Uuid, secret: &str) -> Result<String, anyhow::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
        iat: Utc::now().timestamp() as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(Into::into)
}

// 生成 Subsonic 令牌（MD5(password + salt)）
pub fn generate_subsonic_token(password: &str, salt: &str) -> String {
    let combined = format!("{}{}", password, salt);
    format!("{:x}", md5::compute(combined))
}

// 生成随机 Salt
pub fn generate_salt() -> String {
    rand::thread_rng().gen::<[u8; 16]>()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

// 验证 Subsonic 认证参数
pub fn verify_subsonic_auth(
    provided_token: &str,
    salt: &str,
    password_hash: &str,
) -> Result<bool, anyhow::Error> {
    // 从数据库中的密码哈希还原密码（需要存储明文密码或使用不同的验证方式）
    // 实际实现中，我们可能需要存储密码的原始哈希用于 Subsonic 认证
    Ok(false)
}
```

#### 4.2 认证中间件
```rust
// middleware/auth_middleware.rs
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::HeaderMap,
};
use crate::utils::auth_utils::Claims;
use jsonwebtoken::{decode, Validation, DecodingKey};

pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, anyhow::Error> {
    // 提取认证信息
    let auth_header = headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            // Subsonic 使用查询参数认证
            request.uri().query().and_then(|q| {
                url::form_urlencoded::parse(q.as_bytes())
                    .find(|(k, _)| k == "t")
                    .map(|(_, v)| v.to_string())
            })
        });

    // 验证逻辑...

    Ok(next.run(request).await)
}
```

### 阶段 5: 核心 API 端点实现 (Day 4-8)

#### 5.1 系统端点 (最简单，先实现)
```rust
// handlers/system.rs
use axum::{Router, routing::get, Json};
use crate::models::response::{SubsonicResponse, ResponseContainer};

// GET /rest/ping
pub async fn ping() -> Json<SubsonicResponse<()>> {
    Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: None,
        },
    })
}

// GET /rest/getLicense
pub async fn get_license() -> Json<SubsonicResponse<LicenseResponse>> {
    Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: Some(LicenseResponse {
                valid: true,
                email: "admin@example.com".to_string(),
                key: "licensed".to_string(),
            }),
        },
    })
}

#[derive(Debug, Serialize)]
pub struct LicenseResponse {
    pub valid: bool,
    pub email: String,
    pub key: String,
}

pub fn routes() -> Router {
    Router::new()
        .route("/rest/ping", get(ping))
        .route("/rest/getLicense", get(get_license))
}
```

#### 5.2 浏览类端点
```rust
// handlers/browsing.rs
use axum::{Router, routing::get, extract::Query, Json};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GetIndexesParams {
    pub music_folder_id: Option<i32>,
    pub if_modified_since: Option<i64>,
    pub u: String,  // 用户名
    pub t: Option<String>, // 令牌
    pub s: Option<String>, // salt
    pub p: Option<String>, // 密码
    pub v: String,
    pub c: String,
    pub f: Option<String>, // 格式
}

// GET /rest/getIndexes
pub async fn get_indexes(
    Query(params): Query<GetIndexesParams>,
) -> Result<Json<SubsonicResponse<Indexes>>, AppError> {
    // 1. 验证认证
    // 2. 查询数据库获取艺术家列表
    // 3. 按首字母分组
    // 4. 返回 XML/JSON 格式

    let indexes = Indexes {
        last_modified: Utc::now().timestamp(),
        indexes: vec![],
    };

    Ok(Json(SubsonicResponse {
        response: ResponseContainer {
            status: "ok".to_string(),
            version: "1.16.1".to_string(),
            error: None,
            data: Some(indexes),
        },
    }))
}

// GET /rest/getMusicDirectory
pub async fn get_music_directory(
    Query(params): Query<GetMusicDirectoryParams>,
) -> Result<Json<SubsonicResponse<Directory>>, AppError> {
    // 返回指定目录的内容
}

// GET /rest/getArtist
pub async fn get_artist(
    Query(params): Query<GetArtistParams>,
) -> Result<Json<SubsonicResponse<ArtistDetail>>, AppError> {
    // 返回艺术家详情和专辑列表
}

// GET /rest/getAlbum
pub async fn get_album(
    Query(params): Query<GetAlbumParams>,
) -> Result<Json<SubsonicResponse<AlbumDetail>>, AppError> {
    // 返回专辑详情和歌曲列表
}

// GET /rest/getSong
pub async fn get_song(
    Query(params): Query<GetSongParams>,
) -> Result<Json<SubsonicResponse<Song>>, AppError> {
    // 返回歌曲详情
}

pub fn routes() -> Router {
    Router::new()
        .route("/rest/getIndexes", get(get_indexes))
        .route("/rest/getMusicDirectory", get(get_music_directory))
        .route("/rest/getArtist", get(get_artist))
        .route("/rest/getAlbum", get(get_album))
        .route("/rest/getSong", get(get_song))
}
```

#### 5.3 搜索端点
```rust
// handlers/search.rs
use axum::{Router, routing::get, extract::Query, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Search3Params {
    pub query: String,
    pub artist_count: Option<i32>,
    pub artist_offset: Option<i32>,
    pub album_count: Option<i32>,
    pub album_offset: Option<i32>,
    pub song_count: Option<i32>,
    pub song_offset: Option<i32>,
    // 认证参数...
}

// GET /rest/search3
pub async fn search3(
    Query(params): Query<Search3Params>,
) -> Result<Json<SubsonicResponse<SearchResult3>>, AppError> {
    // 在数据库中搜索匹配的艺术家、专辑、歌曲
    // 使用 SQL LIKE 或全文搜索
}

pub fn routes() -> Router {
    Router::new()
        .route("/rest/search3", get(search3))
        .route("/rest/search2", get(search2))
        .route("/rest/search", get(search))
}
```

#### 5.4 流媒体端点 (核心功能)
```rust
// handlers/streaming.rs
use axum::{
    Router,
    routing::get,
    extract::Query,
    response::{Response, IntoResponse},
    body::StreamBody,
};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct StreamParams {
    pub id: String,
    pub max_bit_rate: Option<i32>,
    pub format: Option<String>, // mp3, flac, wav, aac, m4a, opus, oga
    pub time_offset: Option<i32>,
    pub estimate_content_length: Option<bool>,
    pub converted: Option<bool>,
}

// GET /rest/stream
pub async fn stream(
    Query(params): Query<StreamParams>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 根据 ID 查询歌曲文件路径
    // 2. 检查用户权限
    // 3. 如果需要转码，调用 ffmpeg 或其他工具
    // 4. 返回文件流

    let file_path = get_song_path(&params.id).await?;
    let file = File::open(&file_path).await?;

    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "audio/mpeg".parse().unwrap());

    Ok((headers, body))
}

// GET /rest/download
pub async fn download(
    Query(params): Query<DownloadParams>,
) -> Result<impl IntoResponse, AppError> {
    // 类似 stream，但强制下载
}

pub fn routes() -> Router {
    Router::new()
        .route("/rest/stream", get(stream))
        .route("/rest/download", get(download))
}
```

#### 5.5 播放列表端点
```rust
// handlers/playlist.rs
use axum::{Router, routing::{get, post}, extract::Query, Json};
use serde::Deserialize;

// GET /rest/getPlaylists
pub async fn get_playlists() -> Result<Json<SubsonicResponse<Playlists>>, AppError> {
    // 返回用户的所有播放列表
}

// GET /rest/getPlaylist
pub async fn get_playlist(
    Query(params): Query<GetPlaylistParams>,
) -> Result<Json<SubsonicResponse<PlaylistDetail>>, AppError> {
    // 返回播放列表详情和歌曲列表
}

// POST /rest/createPlaylist
pub async fn create_playlist(
    Query(params): Query<CreatePlaylistParams>,
) -> Result<Json<SubsonicResponse<PlaylistDetail>>, AppError> {
    // 创建新播放列表，可选添加初始歌曲
}

// POST /rest/updatePlaylist
pub async fn update_playlist(
    Query(params): Query<UpdatePlaylistParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 更新播放列表名称、注释、公开状态
    // 添加/删除歌曲
}

// POST /rest/deletePlaylist
pub async fn delete_playlist(
    Query(params): Query<DeletePlaylistParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 删除播放列表
}

pub fn routes() -> Router {
    Router::new()
        .route("/rest/getPlaylists", get(get_playlists))
        .route("/rest/getPlaylist", get(get_playlist))
        .route("/rest/createPlaylist", post(create_playlist))
        .route("/rest/updatePlaylist", post(update_playlist))
        .route("/rest/deletePlaylist", post(delete_playlist))
}
```

#### 5.6 媒体检索端点
```rust
// handlers/media.rs
use axum::{
    Router,
    routing::get,
    extract::Query,
    response::{Response, IntoResponse},
};
use std::path::PathBuf;

// GET /rest/getCoverArt
pub async fn get_cover_art(
    Query(params): Query<GetCoverArtParams>,
) -> Result<impl IntoResponse, AppError> {
    // 1. 根据 ID 获取封面路径（专辑/艺术家）
    // 2. 如果指定了 size，进行图片缩放
    // 3. 返回图片数据

    let cover_path = get_cover_path(&params.id).await?;
    let image_data = tokio::fs::read(cover_path).await?;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "image/jpeg".parse().unwrap());

    Ok((headers, image_data))
}

// GET /rest/getLyrics
pub async fn get_lyrics(
    Query(params): Query<GetLyricsParams>,
) -> Result<Json<SubsonicResponse<Lyrics>>, AppError> {
    // 返回歌曲歌词（可从文件标签或外部文件读取）
}

// GET /rest/getAvatar
pub async fn get_avatar(
    Query(params): Query<GetAvatarParams>,
) -> Result<impl IntoResponse, AppError> {
    // 返回用户头像
}

pub fn routes() -> Router {
    Router::new()
        .route("/rest/getCoverArt", get(get_cover_art))
        .route("/rest/getLyrics", get(get_lyrics))
        .route("/rest/getAvatar", get(get_avatar))
}
```

#### 5.7 库管理端点
```rust
// handlers/library.rs
use axum::{Router, routing::post, extract::Query, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ScrobbleParams {
    pub id: String,
    pub submission: Option<bool>,
    pub time: Option<i64>,
}

// POST /rest/scrobble
pub async fn scrobble(
    Query(params): Query<ScrobbleParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 记录播放历史
    // submission=true: 完整播放
    // submission=false: 正在播放（更新最后播放时间）
}

// POST /rest/star
pub async fn star(
    Query(params): Query<StarParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 收藏艺术家/专辑/歌曲
}

// POST /rest/unstar
pub async fn unstar(
    Query(params): Query<StarParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 取消收藏
}

// POST /rest/setRating
pub async fn set_rating(
    Query(params): Query<SetRatingParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 设置评分（1-5星）
}

// GET /rest/getRating
pub async fn get_rating(
    Query(params): Query<GetRatingParams>,
) -> Result<Json<SubsonicResponse<Rating>>, AppError> {
    // 获取评分
}

// GET /rest/getStarred
pub async fn get_starred() -> Result<Json<SubsonicResponse<Starred>>, AppError> {
    // 获取所有收藏项
}

pub fn routes() -> Router {
    Router::new()
        .route("/rest/scrobble", post(scrobble))
        .route("/rest/star", post(star))
        .route("/rest/unstar", post(unstar))
        .route("/rest/setRating", post(set_rating))
        .route("/rest/getRating", get(get_rating))
        .route("/rest/getStarred", get(get_starred))
}
```

#### 5.8 用户管理端点
```rust
// handlers/user.rs
use axum::{Router, routing::{get, post}, extract::Query, Json};
use serde::Deserialize;

// GET /rest/getUser
pub async fn get_user(
    Query(params): Query<GetUserParams>,
) -> Result<Json<SubsonicResponse<UserResponse>>, AppError> {
    // 返回用户详情（需要管理员权限）
}

// GET /rest/getUsers
pub async fn get_users() -> Result<Json<SubsonicResponse<UsersResponse>>, AppError> {
    // 返回所有用户（需要管理员权限）
}

// POST /rest/createUser
pub async fn create_user(
    Query(params): Query<CreateUserParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 创建新用户（需要管理员权限）
}

// POST /rest/updateUser
pub async fn update_user(
    Query(params): Query<UpdateUserParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 更新用户（需要管理员权限）
}

// POST /rest/deleteUser
pub async fn delete_user(
    Query(params): Query<DeleteUserParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 删除用户（需要管理员权限）
}

// POST /rest/changePassword
pub async fn change_password(
    Query(params): Query<ChangePasswordParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 修改密码
}

pub fn routes() -> Router {
    Router::new()
        .route("/rest/getUser", get(get_user))
        .route("/rest/getUsers", get(get_users))
        .route("/rest/createUser", post(create_user))
        .route("/rest/updateUser", post(update_user))
        .route("/rest/deleteUser", post(delete_user))
        .route("/rest/changePassword", post(change_password))
}
```

#### 5.9 聊天端点
```rust
// handlers/chat.rs
use axum::{Router, routing::{get, post}, extract::Query, Json};
use serde::Deserialize;

// GET /rest/getChatMessages
pub async fn get_chat_messages(
    Query(params): Query<GetChatMessagesParams>,
) -> Result<Json<SubsonicResponse<ChatMessages>>, AppError> {
    // 返回聊天消息（基于时间戳过滤）
}

// POST /rest/addChatMessage
pub async fn add_chat_message(
    Query(params): Query<AddChatMessageParams>,
) -> Result<Json<SubsonicResponse<()>>, AppError> {
    // 添加聊天消息
}

pub fn routes() -> Router {
    Router::new()
        .route("/rest/getChatMessages", get(get_chat_messages))
        .route("/rest/addChatMessage", post(add_chat_message))
}
```

### 阶段 6: 音乐库扫描服务 (Day 8-9)

#### 6.1 扫描服务
```rust
// services/scan_service.rs
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use symphonia::core::probe::Probe;
use symphonia::default::get_probe;
use sqlx::SqlitePool;

pub struct ScanService {
    pool: SqlitePool,
    library_path: PathBuf,
}

impl ScanService {
    pub fn new(pool: SqlitePool, library_path: PathBuf) -> Self {
        Self { pool, library_path }
    }

    pub async fn scan_library(&self) -> Result<ScanResult, anyhow::Error> {
        let mut result = ScanResult::default();

        for entry in WalkDir::new(&self.library_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

            // 支持的音频格式
            if matches!(ext.to_lowercase().as_str(),
                "mp3" | "flac" | "wav" | "m4a" | "aac" | "ogg" | "opus")
            {
                match self.process_audio_file(path).await {
                    Ok(_) => result.successful += 1,
                    Err(e) => {
                        eprintln!("Failed to process {}: {}", path.display(), e);
                        result.failed += 1;
                    }
                }
            }
        }

        Ok(result)
    }

    async fn process_audio_file(&self, path: &Path) -> Result<(), anyhow::Error> {
        // 使用 symphonia 读取元数据
        let probe = get_probe();
        let mut file = std::fs::File::open(path)?;
        let mut format = probe.format(&mut file)?;

        // 提取元数据
        let metadata = format.metadata();

        // 提取轨道信息
        let track = format.default_track().unwrap();

        // 保存到数据库
        self.save_to_database(path, &metadata, track).await?;

        Ok(())
    }

    async fn save_to_database(
        &self,
        path: &Path,
        metadata: &symphonia::core::meta::Metadata,
        track: &symphonia::core::formats::Track,
    ) -> Result<(), anyhow::Error> {
        // 1. 提取艺术家信息
        // 2. 提取专辑信息
        // 3. 提取歌曲信息
        // 4. 保存到数据库（使用 UPSERT 避免重复）

        Ok(())
    }
}

#[derive(Default)]
pub struct ScanResult {
    pub successful: i32,
    pub failed: i32,
}
```

### 阶段 7: 错误处理和响应格式 (Day 9-10)

#### 7.1 错误类型
```rust
// error.rs
use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    // Subsonic 错误码
    MissingParameter(String),
    AuthFailed(String),
    AccessDenied(String),
    NotFound(String),
    ServerBusy(String),

    // 内部错误
    DatabaseError(sqlx::Error),
    IoError(std::io::Error),
    AuthError(anyhow::Error),
    ValidationError(String),
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: String,
    version: String,
    error: SubsonicErrorPayload,
}

#[derive(Debug, Serialize)]
struct SubsonicErrorPayload {
    code: i32,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_code, message) = match self {
            AppError::MissingParameter(msg) => (StatusCode::BAD_REQUEST, 10, msg),
            AppError::AuthFailed(msg) => (StatusCode::UNAUTHORIZED, 30, msg),
            AppError::AccessDenied(msg) => (StatusCode::FORBIDDEN, 40, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, 50, msg),
            AppError::ServerBusy(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 60, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, 0, "Internal server error".to_string()),
        };

        let error_response = ErrorResponse {
            status: "failed".to_string(),
            version: "1.16.1".to_string(),
            error: SubsonicErrorPayload {
                code: error_code,
                message,
            },
        };

        (status_code, Json(error_response)).into_response()
    }
}

// 为各种错误类型实现 From trait
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}
```

### 阶段 8: 主应用和路由配置 (Day 10)

#### 8.1 主应用
```rust
// main.rs
use axum::{Router, middleware};
use std::net::SocketAddr;
use sqlx::sqlite::SqlitePoolOptions;
use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod models;
mod handlers;
mod services;
mod database;
mod middleware;
mod utils;
mod error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载环境变量
    dotenv().ok();

    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 数据库连接
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // 运行迁移
    sqlx::migrate!("./migrations").run(&pool).await?;

    // 创建音乐库扫描服务
    let library_path = std::env::var("MUSIC_LIBRARY_PATH")
        .expect("MUSIC_LIBRARY_PATH must be set");

    let scan_service = services::scan_service::ScanService::new(
        pool.clone(),
        std::path::PathBuf::from(library_path),
    );

    // 构建应用路由
    let app = Router::new()
        // 系统端点
        .merge(handlers::system::routes())
        // 浏览端点
        .merge(handlers::browsing::routes())
        // 搜索端点
        .merge(handlers::search::routes())
        // 流媒体端点
        .merge(handlers::streaming::routes())
        // 播放列表端点
        .merge(handlers::playlist::routes())
        // 媒体检索端点
        .merge(handlers::media::routes())
        // 库管理端点
        .merge(handlers::library::routes())
        // 用户管理端点
        .merge(handlers::user::routes())
        // 聊天端点
        .merge(handlers::chat::routes())
        // 认证中间件
        .layer(middleware::from_fn(middleware::auth_middleware::auth_middleware))
        // CORS
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin("*")
                .allow_methods(vec!["GET", "POST"])
                .allow_headers(tower_http::cors::Any),
        )
        // 日志中间件
        .layer(tower_http::trace::TraceLayer::new_for_http());

    // 启动服务器
    let port = std::env::var("PORT").unwrap_or_else(|_| "4040".to_string());
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let addr = SocketAddr::from(([127, 0, 0, 1], port.parse()?));

    tracing::info!("Server listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

### 阶段 9: 测试 (Day 11-12)

#### 9.1 单元测试
```rust
// tests/unit/auth_test.rs
#[cfg(test)]
mod tests {
    use crate::utils::auth_utils::*;

    #[test]
    fn test_password_hashing() {
        let password = "test123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn test_token_generation() {
        let user_id = Uuid::new_v4();
        let token = generate_token(user_id, "secret").unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_subsonic_token() {
        let password = "password";
        let salt = "salt123";
        let token = generate_subsonic_token(password, salt);
        // MD5("passwordsalt123") = 482c811da5d5b4bc6d497ffa98491e38
        assert_eq!(token, "482c811da5d5b4bc6d497ffa98491e38");
    }
}
```

#### 9.2 集成测试
```rust
// tests/integration/api_test.rs
#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_ping_endpoint() {
        let app = create_test_app().await;

        let request = Request::builder()
            .uri("/rest/ping?u=test&p=test&v=1.16.1&c=test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_search_endpoint() {
        // 测试搜索功能
    }
}
```

### 阶段 10: 部署配置 (Day 12)

#### 10.1 Docker 配置
```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ffmpeg && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/musicflow_server /usr/local/bin
COPY --from=builder /app/migrations /app/migrations
WORKDIR /app
CMD ["musicflow_server"]
```

#### 10.2 docker-compose.yml
```yaml
version: '3.8'

services:
  app:
    build: .
    environment:
      DATABASE_URL: sqlite:/app/data/music_flow.db
      PORT: 4040
      HOST: 0.0.0.0
      MUSIC_LIBRARY_PATH: /music
      RUST_LOG: info
    ports:
      - "4040:4040"
    volumes:
      - ./music:/music
      - ./data:/app/data
```

## API 端点清单（按优先级）

### P0 - 核心功能
- ✅ `ping` - 测试连接
- ✅ `getIndexes` - 获取艺术家索引
- ✅ `getMusicDirectory` - 获取目录内容
- ✅ `getArtist` - 获取艺术家详情
- ✅ `getAlbum` - 获取专辑详情
- ✅ `getSong` - 获取歌曲详情
- ✅ `stream` - 流媒体播放
- ✅ `download` - 文件下载
- ✅ `getCoverArt` - 获取封面
- ✅ `search3` - 搜索

### P1 - 播放列表和收藏
- ✅ `getPlaylists` - 获取播放列表
- ✅ `getPlaylist` - 获取播放列表详情
- ✅ `createPlaylist` - 创建播放列表
- ✅ `updatePlaylist` - 更新播放列表
- ✅ `deletePlaylist` - 删除播放列表
- ✅ `star` / `unstar` - 收藏/取消收藏
- ✅ `getStarred` - 获取收藏项
- ✅ `scrobble` - 播放记录

### P2 - 用户管理
- ✅ `getUser` - 获取用户信息
- ✅ `getUsers` - 获取所有用户
- ✅ `createUser` - 创建用户
- ✅ `updateUser` - 更新用户
- ✅ `deleteUser` - 删除用户
- ✅ `changePassword` - 修改密码

### P3 - 高级功能
- ✅ `getArtistInfo` / `getArtistInfo2` - 艺术家信息
- ✅ `getAlbumList` / `getAlbumList2` - 专辑列表
- ✅ `getRandomSongs` - 随机歌曲
- ✅ `getNowPlaying` - 正在播放
- ✅ `getLyrics` - 歌词
- ✅ `getAvatar` - 头像
- ✅ `setRating` / `getRating` - 评分
- ✅ `getChatMessages` / `addChatMessage` - 聊天
- ✅ `getSystemInfo` - 系统信息
- ✅ `getScanStatus` / `startScan` - 扫描状态

### P4 - 视频和高级流媒体
- ✅ `getVideos` / `getVideoInfo` - 视频
- ✅ `hls` - HLS 流
- ✅ `getLicense` - 许可证信息

## 数据库迁移文件

```sql
-- migrations/001_create_users_table.sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    is_admin BOOLEAN DEFAULT FALSE,
    max_bitrate INTEGER DEFAULT 320,
    download_role BOOLEAN DEFAULT TRUE,
    upload_role BOOLEAN DEFAULT FALSE,
    playlist_role BOOLEAN DEFAULT TRUE,
    cover_art_role BOOLEAN DEFAULT TRUE,
    comment_role BOOLEAN DEFAULT FALSE,
    podcast_role BOOLEAN DEFAULT FALSE,
    share_role BOOLEAN DEFAULT TRUE,
    video_conversion_role BOOLEAN DEFAULT FALSE,
    scrobbling_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_username ON users(username);

-- migrations/002_create_artists_table.sql
CREATE TABLE artists (
    id TEXT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    music_brainz_id VARCHAR(255),
    cover_art_path VARCHAR(500),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_artists_name ON artists(name);

-- migrations/003_create_albums_table.sql
CREATE TABLE albums (
    id TEXT PRIMARY KEY,
    artist_id TEXT REFERENCES artists(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    year INTEGER,
    genre VARCHAR(100),
    cover_art_path VARCHAR(500),
    path VARCHAR(500) NOT NULL,
    song_count INTEGER DEFAULT 0,
    duration INTEGER DEFAULT 0,
    play_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_albums_artist_id ON albums(artist_id);
CREATE INDEX idx_albums_name ON albums(name);

-- migrations/004_create_songs_table.sql
CREATE TABLE songs (
    id TEXT PRIMARY KEY,
    album_id TEXT REFERENCES albums(id) ON DELETE CASCADE,
    artist_id TEXT REFERENCES artists(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    track_number INTEGER,
    disc_number INTEGER,
    duration INTEGER NOT NULL,
    bit_rate INTEGER,
    genre VARCHAR(100),
    year INTEGER,
    content_type VARCHAR(50),
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT,
    play_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_songs_album_id ON songs(album_id);
CREATE INDEX idx_songs_artist_id ON songs(artist_id);
CREATE INDEX idx_songs_title ON songs(title);

-- migrations/005_create_playlists_table.sql
CREATE TABLE playlists (
    id TEXT PRIMARY KEY,
    owner_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    comment TEXT,
    is_public BOOLEAN DEFAULT FALSE,
    song_count INTEGER DEFAULT 0,
    duration INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_playlists_owner_id ON playlists(owner_id);

-- migrations/006_create_playlist_songs_table.sql
CREATE TABLE playlist_songs (
    playlist_id TEXT REFERENCES playlists(id) ON DELETE CASCADE,
    song_id TEXT REFERENCES songs(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (playlist_id, song_id)
);

CREATE INDEX idx_playlist_songs_playlist ON playlist_songs(playlist_id);
CREATE INDEX idx_playlist_songs_song ON playlist_songs(song_id);

-- migrations/007_create_starred_table.sql
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

CREATE INDEX idx_starred_user_artist ON starred(user_id, artist_id);
CREATE INDEX idx_starred_user_album ON starred(user_id, album_id);
CREATE INDEX idx_starred_user_song ON starred(user_id, song_id);

-- migrations/008_create_scrobbles_table.sql
CREATE TABLE scrobbles (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    song_id TEXT REFERENCES songs(id) ON DELETE CASCADE,
    timestamp TIMESTAMP NOT NULL,
    submission BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_scrobbles_user_timestamp ON scrobbles(user_id, timestamp);
CREATE INDEX idx_scrobbles_song ON scrobbles(song_id);

-- migrations/009_create_ratings_table.sql
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

CREATE INDEX idx_ratings_user_artist ON ratings(user_id, artist_id);
CREATE INDEX idx_ratings_user_album ON ratings(user_id, album_id);
CREATE INDEX idx_ratings_user_song ON ratings(user_id, song_id);
```

## 开发工作流

### 1. 环境准备
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 SQLx CLI
cargo install sqlx-cli

# 安装 ffmpeg（用于音频转码）
# Ubuntu: sudo apt-get install ffmpeg
# macOS: brew install ffmpeg
```

### 2. 项目初始化
```bash
# 创建项目
cargo new musicflow_server
cd musicflow_server

# 添加依赖（参考 Cargo.toml）
# 创建目录结构
mkdir -p src/{config,models,schema,handlers,services,database,middleware,utils}
mkdir -p migrations
mkdir -p tests/{unit,integration}

# 设置数据库（SQLite 无需创建数据库，只需指定文件路径）
export DATABASE_URL="sqlite:music_flow.db"

# 运行迁移（会自动创建数据库文件）
sqlx migrate run
```

### 3. 开发流程
```bash
# 开发模式运行
cargo watch -x run

# 运行测试
cargo test

# 检查代码格式
cargo fmt

# 代码检查
cargo clippy

# 生成文档
cargo doc --open
```

### 4. API 测试示例
```bash
# 测试 ping
curl "http://localhost:4040/rest/ping?u=admin&p=admin&v=1.16.1&c=test"

# 测试搜索
curl "http://localhost:4040/rest/search3?u=admin&p=admin&v=1.16.1&c=test&query=love&songCount=10"

# 测试流媒体
curl "http://localhost:4040/rest/stream?u=admin&p=admin&v=1.16.1&c=test&id=<song-id>" --output test.mp3

# 测试封面
curl "http://localhost:4040/rest/getCoverArt?u=admin&p=admin&v=1.16.1&c=test&id=<album-id>&size=300" --output cover.jpg
```

## 性能优化建议

1. **数据库索引**: 确保所有查询字段都有适当索引
2. **连接池**: 使用 SQLx 的连接池，配置合适的连接数
3. **缓存**:
   - 使用 Redis 缓存热门查询结果
   - 缓存封面图片缩放结果
4. **流式处理**:
   - 音频流使用流式传输，避免内存占用
   - 大文件下载使用 chunked 传输
5. **异步扫描**:
   - 音乐库扫描使用后台任务
   - 使用 `tokio::spawn` 并发处理多个文件
6. **CDN**: 封面图片和静态资源使用 CDN 加速

## 安全考虑

1. **认证**:
   - 支持 Subsonic 令牌认证
   - 可选 JWT 认证
   - 密码使用 bcrypt 哈希
2. **授权**:
   - 严格的权限检查
   - 用户只能访问自己的播放列表
   - 管理员权限限制
3. **输入验证**:
   - 所有查询参数验证
   - SQL 注入防护（使用参数化查询）
   - 文件路径遍历防护
4. **速率限制**:
   - 防止 API 滥用
   - 限制登录尝试次数
5. **HTTPS**: 生产环境强制使用 HTTPS

## 扩展功能（未来可选）

1. **Web UI**: 使用 React/Vue 构建前端界面
2. **移动应用**: iOS/Android 客户端
3. **歌词同步**: LRC 格式歌词支持
4. **音乐分析**: BPM、音调分析
5. **推荐系统**: 基于播放历史的推荐
6. **社交功能**: 好友系统、分享
7. **播客支持**: RSS 订阅和下载
8. **智能播放列表**: 基于规则的自动播放列表
9. **Last.fm 集成**: 双向 scrobbling
10. **插件系统**: 支持扩展功能

## 时间估算

- **阶段 1-2**: 1 天（基础搭建 + 数据库设计）
- **阶段 3-4**: 2 天（数据模型 + 认证）
- **阶段 5**: 4 天（核心 API 端点）
- **阶段 6**: 1 天（扫描服务）
- **阶段 7-8**: 1 天（错误处理 + 主应用）
- **阶段 9-10**: 2 天（测试 + 部署配置）
- **总时间**: 约 11-12 天（核心功能完整实现）

## 总结

这是一个完整的 Subsonic API 1.16.1 Rust 服务器实现计划。计划分为 10 个阶段，涵盖了从基础架构到完整功能的所有方面。关键特点：

1. **完整的 API 覆盖**: 支持所有 Subsonic 端点
2. **高性能**: 使用 Axum + Tokio 异步框架
3. **类型安全**: Rust 强类型系统 + SQLx
4. **可扩展**: 模块化设计，易于扩展
5. **生产就绪**: 包含测试、部署、安全考虑

建议按照阶段顺序逐步实现，每个阶段完成后进行测试，确保功能稳定后再进入下一阶段。