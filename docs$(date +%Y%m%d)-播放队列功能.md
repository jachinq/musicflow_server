# 播放队列（书签）功能实现计划

## 任务概述

为 MusicFlow 服务器添加 Subsonic API 的播放队列（书签）功能，包括两个接口：
- `getPlayQueue` - 获取用户的播放队列状态
- `savePlayQueue` - 保存用户的播放队列状态

## 成本评估

### 开发成本：**低到中等**

**工作量估算：**
- 数据库设计与迁移：**1 个任务** (简单表结构)
- Entity 模型：**1 个任务** (参考现有 Playlist 实体)
- Response 模型：**1 个任务** (需要支持 JSON/XML 序列化)
- Service 层：**2 个任务** (两个接口的业务逻辑)
- Handler 层：**2 个任务** (路由处理)
- 路由注册：**1 个任务**
- 测试验证：**1 个任务**

**总计：约 9 个小任务**

### 技术复杂度：**低**

**理由：**
1. **数据模型简单**：播放队列只需要存储歌曲 ID 列表、当前播放歌曲和播放位置
2. **逻辑清晰**：
   - `savePlayQueue` 只是简单的数据写入（INSERT or REPLACE）
   - `getPlayQueue` 只是简单的数据读取 + JOIN 查询歌曲信息
3. **现有基础完善**：
   - 项目已有完整的三层架构（Handler → Service → Entity）
   - 认证中间件已经就绪，可直接使用 `Claims` 获取用户信息
   - 已有类似的 Playlist 实现可供参考
   - Response 序列化机制（JSON/XML）已经完善

### 风险评估：**极低**

1. **无破坏性修改**：完全独立的新功能，不影响现有代码
2. **无复杂依赖**：只依赖现有的 Song 和 User 表
3. **标准化 API**：Subsonic API 规范明确，无需额外设计决策

---

## 实施计划

### 📋 任务清单

#### 1. 数据库设计
- [ ] 创建 `play_queue` 表迁移文件
  - 字段：`id`, `user_id`, `current_song_id`, `position`, `changed_at`, `changed_by`, `updated_at`
- [ ] 创建 `play_queue_songs` 关联表迁移文件
  - 字段：`id`, `play_queue_id`, `song_id`, `song_order`
- [ ] 运行迁移验证表结构

#### 2. Entity 模型层
- [ ] 创建 `src/models/entities/play_queue.rs`
  - `PlayQueue` 实体（主表）
  - `PlayQueueSong` 实体（关联表）
- [ ] 在 `src/models/entities/mod.rs` 中导出新实体

#### 3. Response 模型层
- [ ] 在 `src/models/response/play_queue.rs` 创建响应结构
  - `PlayQueueResponse` 结构体（包含 `current`, `position`, `username`, `changed`, `changedBy`, `entry` 列表）
  - 实现 `Serialize` 和 `ToXml` trait
- [ ] 在 `src/models/response/mod.rs` 中导出

#### 4. Service 层实现
- [ ] 创建 `src/services/play_queue_service.rs`
  - `get_play_queue(user_id: &str)` 方法
  - `save_play_queue(user_id: &str, song_ids: Vec<String>, current: Option<String>, position: Option<i64>, changed_by: &str)` 方法
- [ ] 在 `src/services/mod.rs` 中导出服务

#### 5. Handler 层实现
- [ ] 创建 `src/handlers/play_queue.rs`
  - `get_play_queue` handler（GET `/rest/getPlayQueue`）
  - `save_play_queue` handler（POST/GET `/rest/savePlayQueue`）
  - `PlayQueueState` 状态结构
  - 路由注册函数 `routes()`
- [ ] 在 `src/handlers/mod.rs` 中添加模块声明

#### 6. 路由集成
- [ ] 在 `src/main.rs` 中注册播放队列路由
  - 创建 `PlayQueueState`
  - 合并路由到受保护路由组

#### 7. 测试验证
- [ ] 测试 `savePlayQueue` 接口（创建、更新场景）
- [ ] 测试 `getPlayQueue` 接口（空队列、有数据场景）
- [ ] 验证 JSON 和 XML 响应格式
- [ ] 验证用户隔离（不同用户的队列独立）

---

## 数据库设计详情

### play_queue 表结构
```sql
CREATE TABLE play_queue (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL UNIQUE,  -- 每个用户只有一个播放队列
    current_song_id TEXT,           -- 当前播放的歌曲 ID
    position INTEGER DEFAULT 0,     -- 当前歌曲播放位置（毫秒）
    changed_at TEXT NOT NULL,       -- 最后修改时间（ISO 8601）
    changed_by TEXT NOT NULL,       -- 修改来源客户端
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (current_song_id) REFERENCES songs(id) ON DELETE SET NULL
);
```

### play_queue_songs 关联表
```sql
CREATE TABLE play_queue_songs (
    id TEXT PRIMARY KEY,
    play_queue_id TEXT NOT NULL,
    song_id TEXT NOT NULL,
    song_order INTEGER NOT NULL,    -- 歌曲在队列中的顺序
    FOREIGN KEY (play_queue_id) REFERENCES play_queue(id) ON DELETE CASCADE,
    FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE,
    UNIQUE(play_queue_id, song_order)
);
```

---

## API 接口规范

### getPlayQueue
- **路径**: `/rest/getPlayQueue`
- **方法**: GET
- **参数**: 无（从认证中间件获取用户）
- **响应**:
  - `playQueue` 对象（包含 `current`, `position`, `username`, `changed`, `changedBy`）
  - `entry` 数组（歌曲列表，复用现有 Song 响应结构）

### savePlayQueue
- **路径**: `/rest/savePlayQueue`
- **方法**: GET/POST
- **参数**:
  - `id` (必需，可多个): 歌曲 ID 列表
  - `current` (可选): 当前播放歌曲 ID
  - `position` (可选): 当前播放位置（毫秒）
- **响应**: 空响应（`status="ok"`）

---

## 技术实现要点

### 1. Service 层逻辑

**savePlayQueue：**
```rust
pub async fn save_play_queue(
    &self,
    user_id: &str,
    song_ids: Vec<String>,
    current: Option<String>,
    position: Option<i64>,
    changed_by: &str,
) -> Result<(), AppError>
```
- 使用事务（`transaction()`）确保原子性
- 先删除旧的播放队列和关联歌曲（`DELETE FROM play_queue_songs WHERE play_queue_id = ?`）
- 插入/更新播放队列主记录（`INSERT OR REPLACE`）
- 批量插入歌曲关联（保持顺序）

**getPlayQueue：**
```rust
pub async fn get_play_queue(
    &self,
    user_id: &str,
) -> Result<Option<PlayQueueDetail>, AppError>
```
- 查询 `play_queue` 表获取元数据
- LEFT JOIN `play_queue_songs` 和 `songs` 获取歌曲详情
- 按 `song_order` 排序
- 如果队列不存在返回 `None`

### 2. Handler 层处理

**参数提取：**
```rust
#[derive(Deserialize)]
pub struct SavePlayQueueParams {
    pub id: Vec<String>,           // Axum 自动处理多个相同参数
    pub current: Option<String>,
    pub position: Option<i64>,
}
```

**响应构建：**
- 复用现有的 `Song` 响应结构（`src/models/response/song.rs`）
- 新建 `PlayQueueResponse` 包装器
- 实现 `ToXml` trait（参考 `PlaylistDetailWrapper`）

### 3. 客户端标识

从请求参数中获取 `c`（client name）作为 `changed_by`：
```rust
#[derive(Deserialize)]
pub struct CommonParams {
    pub c: String,  // Subsonic 标准认证参数
}
```

---

## 复杂度对比

| 功能 | 相似度 | 参考实现 |
|------|--------|----------|
| 数据库表设计 | 高 | `playlists` + `playlist_songs` |
| Service 层逻辑 | 高 | `PlaylistService::create_playlist()` |
| Handler 层 | 高 | `handlers/playlist.rs` |
| Response 序列化 | 高 | `PlaylistDetailWrapper` |
| 认证处理 | 完全相同 | 所有受保护端点 |

---

## 预期成果

1. ✅ 用户可以在不同客户端间同步播放队列
2. ✅ 支持保存当前播放位置（精确到毫秒）
3. ✅ 记录最后修改时间和客户端来源
4. ✅ 符合 Subsonic API 1.16.1 规范
5. ✅ 支持 JSON 和 XML 两种响应格式
6. ✅ 完整的用户隔离（每个用户独立队列）

---

## 后续优化（可选）

- [ ] 添加播放队列历史记录功能
- [ ] 支持播放队列的撤销/恢复
- [ ] 添加播放队列统计分析
- [ ] 性能优化：缓存频繁访问的播放队列

---

## 审查章节

_（实施完成后填写）_

### 完成情况
- [ ] 所有任务已完成
- [ ] 测试通过
- [ ] 代码审查通过

### 修改文件清单
_（待实施后填写）_

### 遇到的问题
_（待实施后填写）_

### 性能指标
_（待实施后填写）_
