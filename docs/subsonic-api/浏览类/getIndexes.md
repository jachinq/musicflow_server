# getIndexes

返回所有音乐文件的索引列表。

**参数：**

| 参数              | 必需 | 类型 | 描述                     |
| ----------------- | ---- | ---- | ------------------------ |
| `musicFolderId`   | 否   | 整数 | 按音乐文件夹 ID 过滤     |
| `ifModifiedSince` | 否   | Long | 仅在修改时间戳后返回索引 |

**响应：**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <indexes lastModified="1234567890">
        <index name="A">
            <artist id="123" name="艺术家名称"/>
        </index>
    </indexes>
</subsonic-response>
```

```json
{
  "subsonic-response": {
    "status": "ok",
    "version": "1.16.1",
    "indexes": {
      "lastModified": 1234567890,
      "index": [
        {
          "name": "A",
          "artist": [
            {
              "id": "123",
              "name": "艺术家名称"
            }
          ]
        }
      ]
    }
  }
}
```

