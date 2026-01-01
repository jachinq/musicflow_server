# Music Flow Server

基于 Subsonic API v1.16.1 规范的 Rust 音乐流媒体服务器。

## 🎯 项目概述

这是一个使用 Rust 实现的 Subsonic API 兼容服务器，支持音乐库管理、流媒体播放、播放列表等功能。

可以使用 Subsonic API 客户端或第三方客户端访问，实现音乐流媒体播放、管理等功能。

推荐使用
- [MusicFlow](https://github.com/jachinq/musicflow)： 本项目配套使用的 web 客户端
- [音流播放器](https://devmusic.aqzscn.cn/docs/intro)： 支持 Subsonic 协议的客户端

### 优势

- **安全**： 基于 Rust 语言，安全可靠，无内存泄漏
- **高性能**： 异步 IO 处理，高并发处理能力
- **小体积**： 不到 20M 的打包体积，适合部署到服务器上

### 项目结构
```
musicflow_server/
├── src/
│   ├── main.rs              # 应用入口
│   ├── lib.rs               # 库模块声明
│   ├── config/              # 配置管理
│   ├── database/            # 数据库连接
│   ├── models/              # 数据模型
│   ├── error.rs             # 错误处理
│   ├── utils/               # 工具函数
│   ├── handlers/            # API 处理器
│   ├── services/            # 业务逻辑
│   └── middleware/          # 中间件
├── migrations/              # 数据库迁移
├── .env.example             # 环境变量
├── Cargo.toml               # 依赖配置
└── README.md
```

## 🛠️ 技术栈

- **Web 框架**: Axum 0.7 + Tokio 1
- **数据库**: SQLite 3 + SQLx 0.7
- **认证**: bcrypt + MD5 (Subsonic)
- **音频处理**: Symphonia
- **日志**: tracing
- **序列化**: serde + quick-xml
- **图片处理**: image + webp

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

# 3. 运行数据库迁移(可跳过，启动项目会自动运行)
touch music_flow.db
DATABASE_URL=sqlite:data/music_flow.db
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
DATABASE_URL=sqlite:data/music_flow.db

# 服务器配置
PORT=4040
HOST=127.0.0.1

# 音乐库路径 (必须设置)
MUSIC_LIBRARY_PATH=/path/to/your/music

# 日志级别
RUST_LOG=info
```

### docker 部署

```bash
# 构建 musl 静态链接库
cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/musicflow_server ./docker

# 构建镜像
cd docker
docker build -t musicflow-server .
```


## 📝 API 端点状态

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
- ✅ `getPlaylists` / `getPlaylist`
- ✅ `createPlaylist` / `updatePlaylist` / `deletePlaylist`
- ✅ `star` / `unstar` / `getStarred`
- ✅ `scrobble`

### P2 - 用户管理
- ✅ `getUser` / `getUsers`
- ✅ `createUser` / `updateUser` / `deleteUser`
- ✅ `changePassword`

### P3 - 高级功能
- ✅ `getArtistInfo` / `getAlbumList` / `getRandomSongs`
- ✅ `getNowPlaying` / `getLyrics` / `getAvatar`
- ✅ `setRating` / `getRating`
- ✅ `getChatMessages` / `addChatMessage`
- ✅ `getSystemInfo` / `getScanStatus` / `startScan`

### P4 - 视频和高级流媒体
- ✅ `getVideos` / `getVideoInfo`
- ✅ `hls`
- ✅ `getLicense`

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

## 📚 参考资料

- [Subsonic API 文档](docs/subsonic-api-docs-zh.md) - 中文 API 文档
- [Axum 官方文档](https://docs.rs/axum)
- [SQLx 官方文档](https://docs.rs/sqlx)
- [Subsonic 官方规范](https://www.subsonic.org/pages/api.jsp)

## 📄 许可证

[MIT License](LICENSE)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---