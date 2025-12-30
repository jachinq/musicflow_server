# search3
返回匹配搜索条件的歌曲、艺术家和专辑（包含更多详情）。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `query` | 是 | 字符串 | 搜索查询 |
| `artistCount` | 否 | 整数 | 最大艺术家数量 |
| `artistOffset` | 否 | 整数 | 艺术家偏移量 |
| `albumCount` | 否 | 整数 | 最大专辑数量 |
| `albumOffset` | 否 | 整数 | 专辑偏移量 |
| `songCount` | 否 | 整数 | 最大歌曲数量 |
| `songOffset` | 否 | 整数 | 歌曲偏移量 |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <searchResult3>
        <artist id="123" name="艺术家" coverArt="123" albumCount="5"/>
        <album id="124" name="专辑" artist="艺术家" coverArt="124" songCount="10"/>
        <song id="125" title="歌曲" artist="艺术家" album="专辑" duration="240"/>
    </searchResult3>
</subsonic-response>
```