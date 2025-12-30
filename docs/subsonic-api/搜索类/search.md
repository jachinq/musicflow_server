# search

返回匹配搜索条件的歌曲、艺术家和专辑。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `artist` | 否 | 字符串 | 艺术家过滤 |
| `album` | 否 | 字符串 | 专辑过滤 |
| `title` | 否 | 字符串 | 歌曲标题过滤 |
| `any` | 否 | 字符串 | 搜索所有字段 |
| `count` | 否 | 整数 | 最大结果数 |
| `offset` | 否 | 整数 | 结果偏移量 |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <searchResult>
        <artist id="123" name="艺术家"/>
        <album id="124" name="专辑" artist="艺术家"/>
        <song id="125" title="歌曲" artist="艺术家" album="专辑"/>
    </searchResult>
</subsonic-response>
```