# getTopSongs

返回给定艺术家的热门歌曲，使用来自 last.fm 的数据。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `artist` | 是 | 字符串 | 艺术家名字 |
| `count` | 否 | 整数 | 返回的最大歌曲数量 |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <topSongs>
        <song id="123" title="歌曲标题" artist="艺术家名称" album="专辑名称"
        genre="摇滚" year="2020" duration="240" bitRate="320"
        contentType="audio/mpeg" path="艺术家/专辑/歌曲.mp3"/>
    </topSongs>
</subsonic-response>
```