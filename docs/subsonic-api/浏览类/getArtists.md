# getArtists

与 getIndexes 类似，但按 ID3 标签组织音乐。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `musicFolderId` | 否 | 字符串 | 如果指定了 musicFolderId，则仅返回指定音乐文件夹中的艺术家。请参阅 getMusicFolders。 |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <artists ignoredArticles="The El La Los Las Le Les">
        <index name="A">
            <artist id="6633" name="Aaron Neville" coverArt="ar-6633" albumCount="1"/>
        </index>
        <index name="B">
            <artist id="5950" name="Bob Marley" coverArt="ar-5950" albumCount="8"/>
            <artist id="5957" name="Bruce Dickinson" coverArt="ar-5957" albumCount="2"/>
        </index>
    </artists>
</subsonic-response>
```

```json
{
  "subsonic-response": {
    "status": "ok",
    "version": "1.16.1",
    "artists": {
      "ignoredArticles": "The El La Los Las Le Les",
      "index": [
        {
          "name": "A",
          "artist": [
            {
              "id": "6633",
              "name": "Aaron Neville",
              "coverArt": "ar-6633",
              "albumCount": 1
            }
          ]
        },
        {
          "name": "B",
          "artist": [
            {
              "id": "5950",
              "name": "Bob Marley",
              "coverArt": "ar-5950",
              "albumCount": 8
            },
            {
              "id": "5957",
              "name": "Bruce Dickinson",
              "coverArt": "ar-5957",
              "albumCount": 2
            }
          ]
        }
      ]
    }
  }
}
```