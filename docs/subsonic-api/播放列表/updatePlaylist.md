# updatePlaylist
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