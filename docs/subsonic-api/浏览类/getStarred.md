# getStarred
返回已收藏的项目。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `musicFolderId` | 否 | 整数 | 按音乐文件夹过滤 |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <starred>
        <artist id="123" name="艺术家"/>
        <album id="124" name="专辑" artist="艺术家"/>
        <song id="125" title="歌曲" artist="艺术家" album="专辑"/>
    </starred>
</subsonic-response>
```