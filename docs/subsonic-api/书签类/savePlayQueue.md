# savePlayQueue

保存当前用户播放队列的状态。这包括播放队列中的音乐，当前正在播放的音乐，以及当前音乐中的位置。通常用于允许用户在不同的客户端/应用之间切换，同时保留播放队列的状态（例如，在听音乐时）。

API: `/rest/savePlayQueue.view`

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 歌曲id，支持多个id参数 |
| `current` | 否 | 字符串 | 当前播放歌曲id |
| `position` | 否 | 字符串 | 当前播放歌曲位置 |

**响应：**

返回空的响应数据，status = ok 表示保存成功，否则表示失败。

```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.12.0">
</subsonic-response>
```