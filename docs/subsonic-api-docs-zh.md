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

以下参数可用于大多数端点：

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

## 浏览类端点

### getIndexes
返回所有音乐文件的索引列表。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `musicFolderId` | 否 | 整数 | 按音乐文件夹 ID 过滤 |
| `ifModifiedSince` | 否 | Long | 仅在修改时间戳后返回索引 |

**响应：**
```xml
<indexes lastModified="1234567890">
    <index name="A">
        <artist id="123" name="艺术家名称"/>
    </index>
</indexes>
```

### getMusicDirectory
返回音乐目录中的文件列表。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 目录 ID |

**响应：**
```xml
<directory id="123" name="目录名称">
    <child id="124" title="歌曲" artist="艺术家" album="专辑" isDir="false"/>
</directory>
```

### getArtist
返回艺术家详情。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 艺术家 ID |

**响应：**
```xml
<artist id="123" name="艺术家名称" albumCount="5">
    <album id="124" name="专辑名称" artist="艺术家名称" coverArt="124"/>
</artist>
```

### getAlbum
返回专辑详情。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 专辑 ID |

**响应：**
```xml
<album id="123" name="专辑名称" artist="艺术家名称" coverArt="123" songCount="10">
    <song id="124" title="歌曲标题" artist="艺术家名称" album="专辑名称" duration="240"/>
</album>
```

### getSong
返回歌曲详情。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 歌曲 ID |

**响应：**
```xml
<song id="123" title="歌曲标题" artist="艺术家名称" album="专辑名称"
      genre="摇滚" year="2020" duration="240" bitRate="320"
      contentType="audio/mpeg" path="艺术家/专辑/歌曲.mp3"/>
```

### getVideos
返回所有视频。

**参数：**
无

**响应：**
```xml
<videos>
    <video id="123" title="视频标题" contentType="video/mp4"/>
</videos>
```

### getVideoInfo
返回视频详情。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 视频 ID |

**响应：**
```xml
<videoInfo id="123" title="视频标题">
    <captions id="124" label="英语"/>
</videoInfo>
```

### getArtistInfo
返回艺术家信息和相似艺术家。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 艺术家 ID |
| `count` | 否 | 整数 | 返回的相似艺术家数量 |
| `includeNotPresent` | 否 | 布尔值 | 包含不在库中的艺术家 |

**响应：**
```xml
<artistInfo>
    <biography>艺术家传记文本...</biography>
    <musicBrainzId>mbid</musicBrainzId>
    <lastFmUrl>http://last.fm/...</lastFmUrl>
    <similarArtists>
        <artist id="124" name="相似艺术家"/>
    </similarArtists>
</artistInfo>
```

### getArtistInfo2
返回扩展的艺术家信息。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 艺术家 ID |
| `count` | 否 | 整数 | 相似艺术家数量 |
| `includeNotPresent` | 否 | 布尔值 | 包含不在库中的艺术家 |

**响应：**
与 getArtistInfo 类似，但包含更详细的信息。

### getAlbumList
返回专辑列表。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `type` | 是 | 字符串 | 专辑列表类型：`random`（随机）、`newest`（最新）、`highest`（评分最高）、`frequent`（最常播放）、`recent`（最近播放）、`starred`（已收藏）、`byGenre`（按流派）、`byYear`（按年份） |
| `size` | 否 | 整数 | 返回的专辑数量 |
| `offset` | 否 | 整数 | 跳过的专辑数量 |
| `fromYear` | 否 | 整数 | 起始年份（用于 byYear） |
| `toYear` | 否 | 整数 | 结束年份（用于 byYear） |
| `genre` | 否 | 字符串 | 流派名称（用于 byGenre） |

**响应：**
```xml
<albumList>
    <album id="123" name="专辑名称" artist="艺术家名称" coverArt="123"/>
</albumList>
```

### getAlbumList2
返回包含更多详情的专辑列表。

**参数：**
与 getAlbumList 相同。

**响应：**
```xml
<albumList2>
    <album id="123" name="专辑名称" artist="艺术家名称" artistId="456"
           coverArt="123" songCount="10" created="2020-01-01T00:00:00Z"
           duration="3600" playCount="100" year="2020" genre="摇滚"/>
</albumList2>
```

### getRandomSongs
返回随机歌曲。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `size` | 否 | 整数 | 返回的歌曲数量 |
| `genre` | 否 | 字符串 | 按流派过滤 |
| `fromYear` | 否 | 整数 | 起始年份 |
| `toYear` | 否 | 整数 | 结束年份 |
| `musicFolderId` | 否 | 整数 | 按音乐文件夹过滤 |

**响应：**
```xml
<randomSongs>
    <song id="123" title="歌曲标题" artist="艺术家" album="专辑" duration="240"/>
</randomSongs>
```

### getNowPlaying
返回正在播放的歌曲。

**参数：**
无

**响应：**
```xml
<nowPlaying>
    <entry id="123" title="歌曲" artist="艺术家" username="用户" minutesAgo="5"/>
</nowPlaying>
```

### getStarred
返回已收藏的项目。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `musicFolderId` | 否 | 整数 | 按音乐文件夹过滤 |

**响应：**
```xml
<starred>
    <artist id="123" name="艺术家"/>
    <album id="124" name="专辑" artist="艺术家"/>
    <song id="125" title="歌曲" artist="艺术家" album="专辑"/>
</starred>
```

### getStarred2
返回已收藏的项目（包含更多详情）。

**参数：**
与 getStarred 相同。

**响应：**
```xml
<starred2>
    <artist id="123" name="艺术家" coverArt="123" albumCount="5"/>
    <album id="124" name="专辑" artist="艺术家" coverArt="124" songCount="10"/>
    <song id="125" title="歌曲" artist="艺术家" album="专辑" duration="240"/>
</starred2>
```

---

## 搜索类端点

### search
返回匹配搜索条件的歌曲、艺术家和专辑。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `artist` | 否 | 字符串 | 艺术家过滤 |
| `album` | 否 | 字符串 | 专辑过滤 |
| `title` | 否 | 字符串 | 歌曲标题过滤 |
| `any` | 否 | 字符串 | 搜索所有字段 |
| `count` | 否 | 整数 | 最大结果数 |
| `offset` | 否 | 整数 | 结果偏移量 |

**响应：**
```xml
<searchResult>
    <artist id="123" name="艺术家"/>
    <album id="124" name="专辑" artist="艺术家"/>
    <song id="125" title="歌曲" artist="艺术家" album="专辑"/>
</searchResult>
```

### search2
返回匹配搜索条件的歌曲、艺术家和专辑。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `query` | 是 | 字符串 | 搜索查询 |
| `artistCount` | 否 | 整数 | 最大艺术家数量 |
| `artistOffset` | 否 | 整数 | 艺术家偏移量 |
| `albumCount` | 否 | 整数 | 最大专辑数量 |
| `albumOffset` | 否 | 整数 | 专辑偏移量 |
| `songCount` | 否 | 整数 | 最大歌曲数量 |
| `songOffset` | 否 | 整数 | 歌曲偏移量 |

**响应：**
```xml
<searchResult2>
    <artist id="123" name="艺术家"/>
    <album id="124" name="专辑" artist="艺术家"/>
    <song id="125" title="歌曲" artist="艺术家" album="专辑"/>
</searchResult2>
```

### search3
返回匹配搜索条件的歌曲、艺术家和专辑（包含更多详情）。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `query` | 是 | 字符串 | 搜索查询 |
| `artistCount` | 否 | 整数 | 最大艺术家数量 |
| `artistOffset` | 否 | 整数 | 艺术家偏移量 |
| `albumCount` | 否 | 整数 | 最大专辑数量 |
| `albumOffset` | 否 | 整数 | 专辑偏移量 |
| `songCount` | 否 | 整数 | 最大歌曲数量 |
| `songOffset` | 否 | 整数 | 歌曲偏移量 |

**响应：**
```xml
<searchResult3>
    <artist id="123" name="艺术家" coverArt="123" albumCount="5"/>
    <album id="124" name="专辑" artist="艺术家" coverArt="124" songCount="10"/>
    <song id="125" title="歌曲" artist="艺术家" album="专辑" duration="240"/>
</searchResult3>
```

---

## 流媒体端点

### stream
流式传输歌曲。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 歌曲 ID |
| `maxBitRate` | 否 | 整数 | 最大比特率 |
| `format` | 否 | 字符串 | 输出格式：`mp3`、`flac`、`wav`、`aac`、`m4a`、`opus`、`oga` |
| `timeOffset` | 否 | 整数 | 时间偏移（秒） |
| `size` | 否 | 字符串 | 视频尺寸（用于视频流） |
| `estimateContentLength` | 否 | 布尔值 | 估算内容长度 |
| `converted` | 否 | 布尔值 | 实时转换 |

**响应：**
二进制音频/视频数据

### download
下载歌曲或视频。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 项目 ID |

**响应：**
二进制文件数据

### hls
使用 HLS 流式传输视频。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 视频 ID |
| `bitRate` | 否 | 整数 | 比特率 |
| `audioTrack` | 否 | 字符串 | 音轨 ID |

**响应：**
HLS 播放列表数据

---

## 播放列表端点

### getPlaylists
返回所有播放列表。

**参数：**
无

**响应：**
```xml
<playlists>
    <playlist id="123" name="我的播放列表" owner="用户名" public="false" songCount="10"/>
</playlists>
```

### getPlaylist
返回播放列表详情。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 播放列表 ID |

**响应：**
```xml
<playlist id="123" name="我的播放列表" owner="用户名" public="false" songCount="10" duration="3600">
    <entry id="124" title="歌曲" artist="艺术家" album="专辑" duration="240"/>
</playlist>
```

### createPlaylist
创建新播放列表。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `name` | 是 | 字符串 | 播放列表名称 |
| `songId` | 否 | 数组 | 要添加的歌曲 ID 数组 |

**响应：**
```xml
<playlist id="123" name="我的播放列表" owner="用户名" public="false" songCount="0"/>
```

### updatePlaylist
更新播放列表。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `playlistId` | 是 | 字符串 | 播放列表 ID |
| `name` | 否 | 字符串 | 新名称 |
| `comment` | 否 | 字符串 | 注释 |
| `public` | 否 | 布尔值 | 公开可见性 |
| `songIdToAdd` | 否 | 数组 | 要添加的歌曲 |
| `songIndexToRemove` | 否 | 数组 | 要删除的歌曲索引 |

**响应：**
成功/失败状态

### deletePlaylist
删除播放列表。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 播放列表 ID |

**响应：**
成功/失败状态

---

## 媒体检索端点

### getCoverArt
返回封面图片。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 封面 ID |
| `size` | 否 | 整数 | 期望的尺寸（像素） |

**响应：**
二进制图像数据

### getLyrics
返回歌曲歌词。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `artist` | 否 | 字符串 | 艺术家名称 |
| `title` | 否 | 字符串 | 歌曲标题 |

**响应：**
```xml
<lyrics artist="艺术家" title="歌曲">歌词文本...</lyrics>
```

### getAvatar
返回用户头像。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `username` | 是 | 字符串 | 用户名 |

**响应：**
二进制图像数据

---

## 库管理端点

### scrobble
记录播放（报告播放历史）。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 歌曲 ID |
| `submission` | 否 | 布尔值 | true 为提交播放，false 为正在播放 |
| `time` | 否 | Long | Unix 时间戳（毫秒） |

**响应：**
成功/失败状态

### star
收藏项目。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 否 | 字符串 | 项目 ID |
| `albumId` | 否 | 字符串 | 专辑 ID |
| `artistId` | 否 | 字符串 | 艺术家 ID |

**响应：**
成功/失败状态

### unstar
取消收藏项目。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 否 | 字符串 | 项目 ID |
| `albumId` | 否 | 字符串 | 专辑 ID |
| `artistId` | 否 | 字符串 | 艺术家 ID |

**响应：**
成功/失败状态

### setRating
为项目设置评分。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 项目 ID |
| `rating` | 是 | 整数 | 评分（1-5） |

**响应：**
成功/失败状态

### getRating
获取项目评分。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 项目 ID |

**响应：**
```xml
<rating id="123" rating="5"/>
```

---

## 聊天端点

### getChatMessages
返回聊天消息。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `since` | 否 | Long | Unix 时间戳（毫秒） |

**响应：**
```xml
<chatMessages>
    <message id="123" username="用户" message="你好" time="1234567890"/>
</chatMessages>
```

### addChatMessage
添加聊天消息。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `message` | 是 | 字符串 | 消息文本 |

**响应：**
成功/失败状态

---

## 用户管理端点

### getUser
返回用户详情。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `username` | 是 | 字符串 | 用户名 |

**响应：**
```xml
<user username="用户" email="user@example.com" admin="false"
      scrobblingEnabled="true" maxBitRate="320"
      downloadRole="true" uploadRole="false"
      playlistRole="true" coverArtRole="true"
      commentRole="false" podcastRole="false"
      shareRole="true" videoConversionRole="false"/>
```

### getUsers
返回所有用户。

**参数：**
无

**响应：**
```xml
<users>
    <user username="用户" email="user@example.com" admin="false"/>
</users>
```

### createUser
创建新用户。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `username` | 是 | 字符串 | 用户名 |
| `password` | 是 | 字符串 | 密码 |
| `email` | 是 | 字符串 | 邮箱地址 |
| `ldapAuth` | 否 | 布尔值 | 使用 LDAP 认证 |
| `admin` | 否 | 布尔值 | 管理员权限 |
| `scrobblingEnabled` | 否 | 布尔值 | 启用播放记录 |
| `maxBitRate` | 否 | 整数 | 最大比特率 |
| `downloadRole` | 否 | 布尔值 | 下载权限 |
| `uploadRole` | 否 | 布尔值 | 上传权限 |
| `playlistRole` | 否 | 布尔值 | 播放列表管理 |
| `coverArtRole` | 否 | 布尔值 | 封面管理 |
| `commentRole` | 否 | 布尔值 | 评论权限 |
| `podcastRole` | 否 | 布尔值 | 播客管理 |
| `shareRole` | 否 | 布尔值 | 分享权限 |
| `videoConversionRole` | 否 | 布尔值 | 视频转换 |

**响应：**
成功/失败状态

### updateUser
更新用户。

**参数：**
与 createUser 相同，但 `username` 为必需。

**响应：**
成功/失败状态

### deleteUser
删除用户。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `username` | 是 | 字符串 | 用户名 |

**响应：**
成功/失败状态

### changePassword
修改用户密码。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `username` | 是 | 字符串 | 用户名 |
| `password` | 是 | 字符串 | 新密码 |

**响应：**
成功/失败状态

---

## 系统端点

### ping
测试连接性。

**参数：**
无

**响应：**
```xml
<subsonic-response status="ok" version="1.16.1"/>
```

### getLicense
返回许可证信息。

**参数：**
无

**响应：**
```xml
<license valid="true" email="user@example.com" key="许可证密钥"/>
```

### getSystemInfo
返回系统信息。

**参数：**
无

**响应：**
```xml
<systemInfo>
    <version>1.16.1</version>
    <server>Subsonic</server>
    <type>Subsonic</type>
</systemInfo>
```

### getScanStatus
返回扫描状态。

**参数：**
无

**响应：**
```xml
<scanStatus scanning="false" count="1000"/>
```

### startScan
启动库扫描。

**参数：**
无

**响应：**
```xml
<scanStatus scanning="true" count="0"/>
```

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