# Music Flow Server

基于 Subsonic API 1.16.1 规范的 Rust 音乐流媒体服务器。

## 🎯 项目概述

这是一个使用 Rust 实现的 Subsonic API 兼容服务器，支持音乐库管理、流媒体播放、播放列表等功能。

### 项目结构
```
musicflow_server/
├── src/
│   ├── main.rs              # 应用入口
│   ├── lib.rs               # 库模块声明
│   ├── config/              # 配置管理
│   ├── database/            # 数据库连接
│   ├── models/              # 数据模型 (9个)
│   ├── error.rs             # 错误处理
│   ├── utils/               # 工具函数
│   ├── handlers/            # API 处理器 (待实现)
│   ├── services/            # 业务逻辑 (待实现)
│   └── middleware/          # 中间件 (待实现)
├── migrations/              # 数据库迁移 (9个)
├── .env                     # 环境变量
├── Cargo.toml               # 依赖配置
└── README.md
```

## 🛠️ 技术栈

- **Web 框架**: Axum 0.7 + Tokio 1
- **数据库**: SQLite 3 + SQLx 0.7
- **认证**: bcrypt + JWT + MD5 (Subsonic)
- **音频处理**: Symphonia
- **日志**: tracing
- **序列化**: serde + quick-xml

## 📦 数据库表结构

1. **users** - 用户信息和权限
2. **artists** - 艺术家元数据
3. **albums** - 专辑信息
4. **songs** - 歌曲详情
5. **playlists** - 播放列表
6. **playlist_songs** - 播放列表歌曲关联
7. **starred** - 收藏项
8. **scrobbles** - 播放记录
9. **ratings** - 评分

## 🚀 快速开始

### 环境要求
- Rust 1.70+
- SQLite 3
- SQLx CLI (`cargo install sqlx-cli`)

### 安装和运行

```bash
# 1. 克隆项目
cd musicflow_server

# 2. 设置环境变量
cp .env.example .env
# 编辑 .env 设置 MUSIC_LIBRARY_PATH

# 3. 运行数据库迁移
touch music_flow.db
DATABASE_URL=sqlite:music_flow.db
sqlx migrate run

# 4. 编译项目
cargo build

# 5. 运行服务器
cargo run
```

### 开发

```bash
# 运行测试
cargo test

# 检查代码
cargo check
cargo clippy

# 格式化代码
cargo fmt

# 开发模式（需要 cargo-watch）
cargo watch -x run
```

## 🔧 环境变量

```bash
# 数据库 (SQLite)
DATABASE_URL=sqlite:music_flow.db

# 服务器配置
PORT=4040
HOST=127.0.0.1

# JWT 密钥
JWT_SECRET=your-secret-key-change-in-production

# 音乐库路径 (必须设置)
MUSIC_LIBRARY_PATH=/path/to/your/music

# 日志级别
RUST_LOG=info
```

## 📝 API 端点状态

### P0 - 核心功能 (待实现)
- ❌ `ping` - 测试连接
- ❌ `getIndexes` - 获取艺术家索引
- ❌ `getMusicDirectory` - 获取目录内容
- ❌ `getArtist` - 获取艺术家详情
- ❌ `getAlbum` - 获取专辑详情
- ❌ `getSong` - 获取歌曲详情
- ❌ `stream` - 流媒体播放
- ❌ `download` - 文件下载
- ❌ `getCoverArt` - 获取封面
- ❌ `search3` - 搜索

### P1 - 播放列表和收藏 (待实现)
- ❌ `getPlaylists` / `getPlaylist`
- ❌ `createPlaylist` / `updatePlaylist` / `deletePlaylist`
- ❌ `star` / `unstar` / `getStarred`
- ❌ `scrobble`

### P2 - 用户管理 (待实现)
- ❌ `getUser` / `getUsers`
- ❌ `createUser` / `updateUser` / `deleteUser`
- ❌ `changePassword`

### P3 - 高级功能 (待实现)
- ❌ `getArtistInfo` / `getAlbumList` / `getRandomSongs`
- ❌ `getNowPlaying` / `getLyrics` / `getAvatar`
- ❌ `setRating` / `getRating`
- ❌ `getChatMessages` / `addChatMessage`
- ❌ `getSystemInfo` / `getScanStatus` / `startScan`

### P4 - 视频和高级流媒体 (待实现)
- ❌ `getVideos` / `getVideoInfo`
- ❌ `hls`
- ❌ `getLicense`

## 🔐 认证方式

支持 Subsonic 标准认证：

### 方法 1: 密码认证
```
?u=username&p=password&v=1.16.1&c=clientName
```

### 方法 2: 令牌认证 (推荐)
```
?u=username&t=token&s=salt&v=1.16.1&c=clientName
```
其中 `token = MD5(password + salt)`

## 📋 数据库迁移

```bash
# 查看所有迁移
ls migrations/

# 运行迁移
sqlx migrate run

# 回滚迁移
sqlx migrate revert

# 创建新迁移
sqlx migrate add <migration_name>
```

## 🔍 数据库查询示例

```bash
# 查看数据库内容
sqlite3 music_flow.db

sqlite> .tables
sqlite> SELECT * FROM users;
sqlite> .schema users
```

## 🎯 下一步计划

### 阶段 2: 核心 API 端点
实现 P0 级别的 10 个核心 API 端点，包括：
- 系统端点 (ping, getLicense)
- 浏览端点 (getIndexes, getArtist, getAlbum, getSong)
- 搜索端点 (search3)
- 流媒体端点 (stream, download)
- 媒体检索端点 (getCoverArt)

### 阶段 3: 音乐库扫描
实现音乐库扫描服务，自动导入音频文件的元数据。

### 阶段 4: 高级功能
实现播放列表、收藏、评分、用户管理等功能。

## 🐛 已知问题

1. **编译时警告** - 大量未使用的代码警告，这是正常的，因为很多功能还未实现
2. **文档测试失败** - 需要修复文档示例
3. **音乐库路径** - 需要手动设置有效的音乐库路径

## 📚 参考资料

- [Subsonic API 文档](docs/subsonic-api-docs-zh.md) - 中文 API 文档
- [Axum 官方文档](https://docs.rs/axum)
- [SQLx 官方文档](https://docs.rs/sqlx)
- [Subsonic 官方规范](https://www.subsonic.org/pages/api.jsp)

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

**当前状态**: ✅ 阶段 1 完成 - 基础架构已就绪，等待实现 API 端点