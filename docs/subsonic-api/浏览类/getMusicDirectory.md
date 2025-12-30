# getMusicDirectory

返回音乐目录中的文件列表。

**参数：**

| 参数 | 必需 | 类型   | 描述    |
| ---- | ---- | ------ | ------- |
| `id` | 是   | 字符串 | 目录 ID |

**响应：**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <directory id="123" name="目录名称">
        <child id="124" title="歌曲" artist="艺术家" album="专辑" isDir="false"/>
    </directory>
</subsonic-response>
```

```json
{
  "subsonic-response": {
    "status": "ok",
    "version": "1.16.1",
    "directory": {
      "id": "123",
      "name": "目录名称",
      "child": [
        {
          "id": "124",
          "title": "歌曲",
          "artist": "艺术家",
          "album": "专辑",
          "isDir": false
        }
      ]
    }
  }
}
```
