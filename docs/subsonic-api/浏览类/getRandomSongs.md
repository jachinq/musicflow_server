# getRandomSongs
返回随机歌曲。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `size` | 否 | 整数 | 返回的歌曲数量 |
| `genre` | 否 | 字符串 | 按流派过滤 |
| `fromYear` | 否 | 整数 | 起始年份 |
| `toYear` | 否 | 整数 | 结束年份 |
| `musicFolderId` | 否 | 整数 | 按音乐文件夹过滤 |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <randomSongs>
        <song id="123" title="歌曲标题" artist="艺术家" album="专辑" duration="240"/>
    </randomSongs>
</subsonic-response>
```