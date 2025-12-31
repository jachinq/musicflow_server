# getSong
返回歌曲详情。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 歌曲 ID |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <song id="123" title="歌曲标题" artist="艺术家名称" album="专辑名称"
        genre="摇滚" year="2020" duration="240" bitRate="320"
        contentType="audio/mpeg" path="艺术家/专辑/歌曲.mp3" userRating="5"/>
</subsonic-response>
```

```json
{
  "subsonic-response": {
    "status": "ok",
    "version": "1.16.1",
    "song": {
        "id": "123",
        "title": "歌曲标题",
        "artist": "艺术家名称",
        "album": "专辑名称",
        "genre": "摇滚",
        "year": 2020,
        "duration": 240,
        "bitRate": 320,
        "contentType": "audio/mpeg",
        "path": "艺术家/专辑/歌曲.mp3",
        "userRating": 5
    }
  }
}
```