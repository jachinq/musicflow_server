# getGenres

返回流派列表。

**参数：**
无

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <genres>
        <genre songCount="28" albumCount="6">Electronic</genre>
    </genres>
</subsonic-response>
```

```json
{
  "subsonic-response": {
    "status": "ok",
    "version": "1.16.1",
    "genres": {
      "genre": [
        {
          "songCount": 28,
          "albumCount": 6,
          "value": "Electronic"
        }
      ]
    }
  }
}
```