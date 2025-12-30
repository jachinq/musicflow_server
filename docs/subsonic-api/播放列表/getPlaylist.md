# getPlaylist

返回播放列表详情。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 播放列表 ID |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <playlist id="123" name="我的播放列表" owner="用户名" public="false" songCount="10" duration="3600">
        <entry id="124" title="歌曲" artist="艺术家" album="专辑" duration="240"/>
    </playlist>
</subsonic-response>
```