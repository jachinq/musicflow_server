# createPlaylist
创建新播放列表。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `name` | 是 | 字符串 | 播放列表名称 |
| `songId` | 否 | 数组 | 要添加的歌曲 ID 数组 |

**响应：**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
	<playlist id="123" name="我的播放列表" owner="用户名" public="false" songCount="0"/>
</subsonic-response>
```