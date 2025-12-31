# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 Rust 实现的 Subsonic API v1.16.1 兼容音乐流媒体服务器,支持音乐库扫描、流媒体播放、播放列表管理等功能。

## 常用命令

### 开发和测试
```bash
# 检查代码(推荐用于快速验证)
cargo check

# 运行测试
cargo test

# 运行特定测试
cargo test test_name

# 代码风格检查
cargo clippy

# 格式化代码
cargo fmt
```

### 构建和运行
```bash
# 开发模式运行
cargo run

# 发布构建
cargo build --release

# 带自动重载的开发模式(需要 cargo-watch)
cargo watch -x run
```

### 数据库管理
```bash
# 查看数据库表结构
sqlite3 music_flow.db ".schema"

# 查看所有表
sqlite3 music_flow.db ".tables"

# 运行迁移(通常不需要手动执行,启动时自动运行)
sqlx migrate run

# 回滚迁移
sqlx migrate revert

# 创建新迁移
sqlx migrate add <migration_name>
```

## 核心架构

### 三层架构设计

1. **Handlers 层** ([src/handlers/](src/handlers/))
   - 处理 HTTP 请求和响应
   - 参数提取和验证
   - 调用 Services 层
   - 返回 `ApiResponse<T>` 自动序列化为 JSON/XML

2. **Services 层** ([src/services/](src/services/))
   - 业务逻辑实现
   - 通过 `ServiceContext` 管理数据库连接和事务
   - 返回 `Result<T, AppError>`
   - 关键服务:
     - `BrowsingService`: 音乐浏览(艺术家/专辑/歌曲)
     - `SearchService`: 搜索功能
     - `PlaylistService`: 播放列表管理
     - `ScanService`: 音乐库扫描
     - `LibraryService`: 收藏/评分/Scrobble
     - `UserService`: 用户管理
     - `AuthService`: 认证服务

3. **Models 层** ([src/models/](src/models/))
   - `entities/`: 数据库实体(对应表结构)
   - `dto/`: 数据传输对象(中间层转换)
   - `response/`: API 响应结构(支持 JSON 和 XML 序列化)

### 关键设计模式

#### ServiceContext 和事务管理
所有 Service 通过 `ServiceContext` 访问数据库:
```rust
pub struct ServiceContext {
    pub pool: SqlitePool,
}
```

使用 `transaction()` 方法执行数据库事务,自动处理提交和回滚:
```rust
service.ctx.transaction(|tx| async move {
    sqlx::query("...").execute(&mut **tx).await?;
    Ok(result)
}.boxed()).await?;
```

#### ID 生成策略
使用 `id_builder` 模块生成唯一 ID:
- `generate_id()`: 默认 16 字符,基于 UUID v4
- `CoverArt::get_id(id)`: 封面 ID, 格式：`al-{专辑id}` 或 `ar-{艺术家id}`，和实际的专辑/艺术家 ID 一致，只是加了不同前缀

#### 认证机制
通过 `auth_middleware` 中间件实现 Subsonic 标准认证:
1. **密码认证**: `?u=username&p=password`
2. **Token 认证**(推荐): `?u=username&t=token&s=salt` 其中 `token = MD5(password + salt)`

认证信息存储在 `Claims` 结构中,通过 `req.extensions` 传递:
```rust
pub struct Claims {
    pub sub: String,      // user_id
    pub username: String,
    pub is_admin: bool,
    pub exp: usize,
    pub iat: usize,
}
```

#### 响应格式处理
使用 `ApiResponse<T>` 包装器自动处理 JSON/XML 序列化:
```rust
ApiResponse::ok(Some(data), format)
```

格式通过 `Format` extractor 从查询参数 `f` 提取,默认为 XML(Subsonic 标准)。

#### 错误处理
使用 `AppError` 枚举统一错误处理,自动转换为 Subsonic 错误响应:
- `MissingParameter`: 缺少参数(错误码 10)
- `AuthFailed`: 认证失败(错误码 30)
- `AccessDenied`: 访问被拒绝(错误码 40)
- `NotFound`: 资源未找到(错误码 50)
- `ServerBusy`: 服务器繁忙(错误码 60)

### 音乐库扫描流程

`ScanService` 负责扫描音乐库:
1. 使用 `walkdir` 遍历音乐目录
2. 使用 `symphonia` 提取音频元数据
3. 使用 `pinyin` 库生成中文拼音索引
4. 使用 `image` 库处理封面图片(支持 WebP 转换)
5. 批量写入数据库(artists → albums → songs)

### 数据库表关系

```
users (用户)
  │
  ├─> playlists (播放列表)
  │     └─> playlist_songs (播放列表歌曲)
  │           └─> songs
  │
  ├─> starred (收藏)
  ├─> ratings (评分)
  └─> scrobbles (播放记录)

artists (艺术家)
  └─> albums (专辑)
        └─> songs (歌曲)
```

## 重要约定

### 新增 API 端点
1. 在对应的 `handlers/` 模块中添加处理函数
2. 在 `services/` 中实现业务逻辑
3. 在 `models/response/` 中定义响应结构(实现 `Serialize` 和 `ToXml`)
4. 在 `routes()` 函数中注册路由
5. 确保需要认证的端点合并到 `protected_routes`

### 数据库操作
- 简单查询直接使用 `pool`
- 涉及多表操作使用 `ServiceContext::transaction()`
- 使用 `sqlx::query_as::<_, Entity>` 映射到实体
- 避免 N+1 查询,使用 JOIN 或批量查询

### 密码处理
目前使用明文密码存储(开发阶段),生产环境需要:
1. 使用 `bcrypt::hash()` 加密密码
2. 使用 `bcrypt::verify()` 验证密码
3. 更新 `authenticate_with_password()` 和 `authenticate_subsonic()` 逻辑

### 日志记录
- 使用 `tracing::info!`, `tracing::debug!`, `tracing::error!` 等宏
- 通过 `RUST_LOG` 环境变量控制日志级别
- 数据库错误、IO 错误等已在 `AppError::into_response()` 中自动记录

### 测试要求
- 为 Service 层编写单元测试
- 使用 `sqlite::memory:` 作为测试数据库
- 测试事务回滚场景
- 参考 `services/context.rs` 中的测试示例

## 技术栈特性

- **Axum 0.7**: 基于 Tower 的 Web 框架,使用 extractors 和 middleware 模式
- **SQLx 0.7**: 编译时检查 SQL,使用 `sqlx::query_as!` 需要 `DATABASE_URL` 环境变量
- **Symphonia**: 支持 MP3, AAC, FLAC, OGG, WAV 等音频格式
- **Tower-HTTP**: 提供 CORS, Trace, ServeDir 等中间件

## 常见任务

### 添加新的数据库字段
1. 在 `migrations/` 创建新迁移: `sqlx migrate add add_field_to_table`
2. 编写 SQL DDL 语句
3. 更新对应的 `models/entities/` 结构体
4. 更新相关的 Service 和 Handler 逻辑
5. 测试迁移: `sqlx migrate run`

### 支持新的音频格式
1. 在 `Cargo.toml` 的 `symphonia` features 中添加格式
2. 更新 `ScanService` 的文件扩展名过滤逻辑
3. 测试元数据提取

### 优化查询性能
1. 使用 `EXPLAIN QUERY PLAN` 分析 SQL
2. 在 migrations 中添加索引
3. 考虑使用批量查询替代循环查询
4. 对热点查询添加 `tracing::debug!` 记录执行时间
