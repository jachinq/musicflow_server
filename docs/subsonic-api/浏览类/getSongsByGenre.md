# getSongsByGenre

返回给定流派的歌曲。


**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `genre` | 是 | 字符串 | 流派名称 |
| `count` | 否 | 整数 | 最大歌曲数量 |
| `offset` | 否 | 整数 | 结果偏移量 |
| `musicFolderId` | 否 | 整数 | 按音乐文件夹过滤 |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <songsByGenre>
        <song id="123" title="歌曲标题" artist="艺术家名称" album="专辑名称"
        genre="摇滚" year="2020" duration="240" bitRate="320"
        contentType="audio/mpeg" path="艺术家/专辑/歌曲.mp3"/>
    </songsByGenre>
</subsonic-response>
```