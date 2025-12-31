# Subsonic API 中文文档

完整的 Subsonic REST API 参考文档。本文档涵盖了所有可用的端点、参数、认证方式和响应格式。

---

## 目录

1. [概述](#概述)
2. [认证方式](#认证方式)
3. [通用参数](#通用参数)
4. [响应格式](#响应格式)
5. [错误处理](#错误处理)
6. [浏览类端点](#浏览类端点)
7. [搜索类端点](#搜索类端点)
8. [流媒体端点](#流媒体端点)
9. [播放列表端点](#播放列表端点)
10. [媒体检索端点](#媒体检索端点)
11. [库管理端点](#库管理端点)
12. [聊天端点](#聊天端点)
13. [用户管理端点](#用户管理端点)
14. [系统端点](#系统端点)
15. [使用示例](#使用示例)

---

## 概述

Subsonic API 是一个 RESTful API，允许你通过编程方式与 Subsonic 媒体服务器进行交互。所有 API 端点遵循以下模式：

```
http://你的服务器.com/rest/[端点名称]
```

### 基础 URL
```
http://你的服务器.com/rest/
```

### API 版本
当前 API 版本：**1.16.1**

---

## 认证方式

所有 API 请求都需要认证。你可以使用以下两种方法之一进行认证：

### 方法 1：用户名/密码（传统方式）
```
?u=username&p=password&v=1.16.1&c=clientName
```

### 方法 2：基于令牌（推荐）
```
?u=username&t=token&s=salt&v=1.16.1&c=clientName
```

其中：
- `t` = MD5(密码 + salt)
- `salt` = 随机字符串（每个请求必须唯一）

### 认证参数

| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `u` | 是 | 字符串 | 用户名 |
| `p` | 否 | 字符串 | 密码（URL编码） |
| `t` | 否 | 字符串 | 令牌（密码 + salt 的 MD5） |
| `s` | 否 | 字符串 | Salt（随机字符串） |
| `v` | 是 | 字符串 | API 版本（例如 "1.16.1"） |
| `c` | 是 | 字符串 | 客户端名称（应用程序标识符） |

---

## 通用参数

以下参数可用于大多数接口：

| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `callback` | 否 | 字符串 | JSONP 回调函数名（用于 JSON 响应） |
| `f` | 否 | 字符串 | 响应格式：`xml`（默认）或 `json` |
| `id` | 是 | 字符串 | 媒体 ID（艺术家、专辑、歌曲等） |
| `limit` | 否 | 整数 | 返回结果的最大数量 |
| `offset` | 否 | 整数 | 跳过的结果数量 |
| `year` | 否 | 整数 | 按年份过滤 |
| `genre` | 否 | 字符串 | 按流派过滤 |
| `from` | 否 | Long | 开始日期的 Unix 时间戳（毫秒） |
| `to` | 否 | Long | 结束日期的 Unix 时间戳（毫秒） |

---

## 响应格式

### XML 响应（默认）

```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <!-- 响应数据 -->
</subsonic-response>
```


### JSON 响应
```json
{
  "subsonic-response": {
    "status": "ok",
    "version": "1.16.1",
    "data": {
      // 响应数据
    }
  }
}
```

### 成功响应
- `status="ok"` - 请求成功
- 包含响应数据

### 错误响应
- `status="failed"` - 请求失败
- 包含错误详情的 `error` 元素/对象

---

## 错误处理

### 错误代码

| 代码 | HTTP 状态 | 描述 |
|------|-----------|------|
| `0` | 200 | 通用错误 |
| `10` | 401 | 缺少必需参数 |
| `20` | 401 | API 版本不兼容 |
| `30` | 401 | 认证失败 |
| `40` | 403 | 权限被拒绝 |
| `50` | 404 | 资源未找到 |
| `60` | 500 | 服务器繁忙 |
| `70` | 500 | 服务器离线 |

### 错误响应格式
```xml
<subsonic-response status="failed" version="1.16.1">
    <error code="40" message="认证失败"/>
</subsonic-response>
```

```json
{
  "subsonic-response": {
    "status": "failed",
    "version": "1.16.1",
    "error": {
      "code": 40,
      "message": "认证失败"
    }
  }
}
```

---

## 具体 API 接口

所有具体的请求接口都是 `/rest/xxx` 的方式。

具体接口对应的请求参数和响应格式，都在 `./subsonic-api` 路径下，对应的具体分类文件夹中。

例如 `/rest/getSongs ` 接口的具体请求参数和响应格式位于 `./subsonic-api/浏览类/getSongs.md`

### 浏览类

- getIndexes 返回所有音乐文件的索引列表
- getMusicDirectory 返回音乐目录中的文件列表
- getGenres 返回流派列表
- getArtists 返回艺术家列表，与 getIndexes 类似，但按 ID3 标签组织音乐。
- getArtist 返回艺术家详情
- getAlbum 返回专辑详情
- getSong 返回歌曲详情
- getVideos 返回所有视频
- getArtistInfo 返回艺术家信息和相似艺术家
- getArtistInfo2 返回扩展的艺术家信息
- getAlbumList 返回专辑列表
- getTopSongs 返回给定艺术家的热门歌曲
- getAlbumList2 返回包含更多详情的专辑列表
- getRandomSongs 返回随机歌曲
- getSongsByGenre 返回给定流派的歌曲
- getNowPlaying 返回正在播放的歌曲
- getStarred 返回已收藏的项目
- getStarred2 返回已收藏的项目（包含更多详情）
- getSimilarSongs2 返回给定歌曲的相似歌曲

### 搜索类

- search 返回匹配搜索条件的歌曲、艺术家和专辑
- search2 返回匹配搜索条件的歌曲、艺术家和专辑
- search3 返回匹配搜索条件的歌曲、艺术家和专辑

---

### 流媒体

- stream 返回流媒体文件
- download 返回下载文件
- hls 流式传输文件

---

### 播放列表

- getPlaylists 返回所有播放列表
- getPlaylist 返回播放列表详情
- createPlaylist 创建新播放列表
- updatePlaylist 更新播放列表
- deletePlaylist 删除播放列表

---

### 媒体检索

- getCoverArt 返回封面图片
- getLyrics 返回歌曲歌词
- getAvatar 返回用户头像
---

### 库管理

- scrobble 记录播放（报告播放历史）
- star 收藏项目
- unstar 取消收藏项目
- setRating 为项目设置评分
- getRating 获取项目评分

---

### 聊天端点

- getChatMessages 返回聊天消息
- addChatMessage 添加聊天消息

---

### 用户管理

- getUser 返回用户详情
- getUsers 返回所有用户
- createUser 创建新用户
- updateUser 更新用户
- deleteUser 删除用户
- changePassword 修改用户密码

---

### 系统

- ping 测试连接性
- getLicense 返回许可证信息
- getSystemInfo 返回系统信息
- getScanStatus 返回扫描状态
- startScan 启动库扫描

---

## 使用示例

### 示例 1：认证和基本请求

**请求（令牌认证）：**
```bash
# 生成令牌
PASSWORD="我的密码"
SALT="随机Salt123"
TOKEN=$(echo -n "${PASSWORD}${SALT}" | md5sum | cut -d' ' -f1)

# 发送请求
curl "http://localhost:4040/rest/ping?u=username&t=${TOKEN}&s=${SALT}&v=1.16.1&c=myApp"
```

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1"/>
```

### 示例 2：获取所有艺术家

**请求：**
```bash
curl "http://localhost:4040/rest/getIndexes?u=username&p=password&v=1.16.1&c=myApp"
```

**响应：**
```xml
<subsonic-response status="ok" version="1.16.1">
    <indexes lastModified="1234567890">
        <index name="A">
            <artist id="123" name="艺术家A"/>
            <artist id="124" name="艺术家B"/>
        </index>
        <index name="B">
            <artist id="125" name="艺术家C"/>
        </index>
    </indexes>
</subsonic-response>
```

### 示例 3：搜索歌曲

**请求：**
```bash
curl "http://localhost:4040/rest/search3?u=username&p=password&v=1.16.1&c=myApp&query=love&songCount=10"
```

**响应：**
```xml
<subsonic-response status="ok" version="1.16.1">
    <searchResult3>
        <song id="123" title="Love Song" artist="艺术家A" album="专辑X"
              duration="240" bitRate="320" coverArt="123"/>
        <song id="124" title="Love Will Tear Us Apart" artist="艺术家B" album="专辑Y"
              duration="300" bitRate="320" coverArt="124"/>
    </searchResult3>
</subsonic-response>
```

### 示例 4：流式传输歌曲

**请求：**
```bash
# 获取流 URL
STREAM_URL="http://localhost:4040/rest/stream?u=username&p=password&v=1.16.1&c=myApp&id=123"

# 使用 mpv 播放或下载
mpv "$STREAM_URL"
```

### 示例 5：创建播放列表

**请求：**
```bash
curl -X POST "http://localhost:4040/rest/createPlaylist" \
  -d "u=username&p=password&v=1.16.1&c=myApp&name=我的最爱&songId=123&songId=124&songId=125"
```

**响应：**
```xml
<subsonic-response status="ok" version="1.16.1">
    <playlist id="123" name="我的最爱" owner="用户名" public="false" songCount="3"/>
</subsonic-response>
```

### 示例 6：获取封面图片

**请求：**
```bash
curl "http://localhost:4040/rest/getCoverArt?u=username&p=password&v=1.16.1&c=myApp&id=123&size=300" \
  --output cover.jpg
```

**响应：**
二进制 JPEG 图像数据

### 示例 7：记录播放历史

**请求：**
```bash
# 报告正在播放
curl "http://localhost:4040/rest/scrobble?u=username&p=password&v=1.16.1&c=myApp&id=123&submission=false"

# 报告播放完成
curl "http://localhost:4040/rest/scrobble?u=username&p=password&v=1.16.1&c=myApp&id=123&submission=true&time=$(date +%s)000"
```

### 示例 8：收藏专辑

**请求：**
```bash
curl "http://localhost:4040/rest/star?u=username&p=password&v=1.16.1&c=myApp&albumId=123"
```

### 示例 9：获取用户信息

**请求：**
```bash
curl "http://localhost:4040/rest/getUser?u=username&p=password&v=1.16.1&c=myApp&username=targetUser"
```

**响应：**
```xml
<subsonic-response status="ok" version="1.16.1">
    <user username="目标用户" email="user@example.com" admin="false"
          scrobblingEnabled="true" maxBitRate="320" downloadRole="true"
          uploadRole="false" playlistRole="true" coverArtRole="true"
          commentRole="false" podcastRole="false" shareRole="true"
          videoConversionRole="false"/>
</subsonic-response>
```

### 示例 10：JSON 响应格式

**请求：**
```bash
curl "http://localhost:4040/rest/ping?u=username&p=password&v=1.16.1&c=myApp&f=json"
```

**响应：**
```json
{
  "subsonic-response": {
    "status": "ok",
    "version": "1.16.1"
  }
}
```

---

## 最佳实践

1. **始终使用令牌认证** - 比明文密码更安全
2. **缓存响应** - 减少频繁访问数据的服务器负载
3. **优雅处理错误** - 检查状态和错误代码
4. **使用分页** - 对于大型数据集，使用 offset 和 limit
5. **尊重速率限制** - 不要发出过多请求
6. **关闭连接** - 正确处理 HTTP 连接
7. **使用 HTTPS** - 当可用时，提高安全性

---

## 附加资源

- [Subsonic 官方网站](https://www.subsonic.org/)
- [Subsonic API 页面](https://www.subsonic.org/pages/api.jsp)
- [Subsonic 论坛](https://forum.subsonic.org/)
- [Subsonic GitHub](https://github.com/subsonic/subsonic)

---

*本文档根据官方 Subsonic API 规范整理，涵盖了 API 版本 1.16.1 中的所有可用端点。*