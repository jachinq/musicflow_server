# getArtist
返回艺术家详情。

**参数：**
| 参数 | 必需 | 类型   | 描述      |
| ---- | ---- | ------ | --------- |
| `id` | 是   | 字符串 | 艺术家 ID |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <artist id="123" name="艺术家名称" albumCount="5">
        <album id="124" name="专辑名称" artist="艺术家名称" coverArt="124"/>
    </artist>
</subsonic-response>
```

```json
{
  "subsonic-response": {
    "status": "ok",
    "version": "1.16.1",
    "artist": {
      "id": "123",
      "name": "艺术家名称",
      "albumCount": 5,
      "album": [
        {
          "id": "124",
          "name": "专辑名称",
          "artist": "艺术家名称",
          "coverArt": "124"
        }
      ]
    }
  }
}
```