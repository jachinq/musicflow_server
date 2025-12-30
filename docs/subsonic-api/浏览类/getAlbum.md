# getAlbum
返回专辑详情。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 专辑 ID |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <album id="123" name="专辑名称" artist="艺术家名称" coverArt="123" songCount="10">
        <song id="124" title="歌曲标题" artist="艺术家名称" album="专辑名称" duration="240"/>
    </album>
</subsonic-response>
```

```json
{
  "subsonic-response": {
    "status": "ok",
    "version": "1.16.1",
    "album": {
      "id": "123",
      "name": "专辑名称",
      "artist": "艺术家名称",
      "coverArt": "123",
      "songCount": 10,
      "song": [
        {
          "id": "124",
          "title": "歌曲标题",
          "artist": "艺术家名称",
          "album": "专辑名称",
          "duration": 240
        }
      ]
    }
  }
}
```