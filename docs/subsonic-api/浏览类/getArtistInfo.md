# getArtistInfo
返回艺术家信息和相似艺术家。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 艺术家 ID |
| `count` | 否 | 整数 | 返回的相似艺术家数量 |
| `includeNotPresent` | 否 | 布尔值 | 包含不在库中的艺术家 |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <artistInfo>
        <biography>艺术家传记文本...</biography>
        <musicBrainzId>mbid</musicBrainzId>
        <lastFmUrl>http://last.fm/...</lastFmUrl>
        <similarArtists>
            <artist id="124" name="相似艺术家"/>
        </similarArtists>
    </artistInfo>
</subsonic-response>
```