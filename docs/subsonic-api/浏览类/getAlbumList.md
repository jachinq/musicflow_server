# getAlbumList

返回专辑列表。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `type` | 是 | 字符串 | 专辑列表类型：`random`（随机）、`newest`（最新）、`highest`（评分最高）、`frequent`（最常播放）、`recent`（最近播放）、`starred`（已收藏）、`byGenre`（按流派）、`byYear`（按年份） |
| `size` | 否 | 整数 | 返回的专辑数量 |
| `offset` | 否 | 整数 | 跳过的专辑数量 |
| `fromYear` | 否 | 整数 | 起始年份（用于 byYear） |
| `toYear` | 否 | 整数 | 结束年份（用于 byYear） |
| `genre` | 否 | 字符串 | 流派名称（用于 byGenre） |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <albumList>
        <album id="123" name="专辑名称" artist="艺术家名称" coverArt="123"/>
    </albumList>
</subsonic-response>
```