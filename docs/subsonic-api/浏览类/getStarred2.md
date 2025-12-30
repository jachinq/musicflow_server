# getStarred2
返回已收藏的项目（包含更多详情）。

**参数：**
与 getStarred 相同。

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <starred2>
        <artist id="123" name="艺术家" coverArt="123" albumCount="5"/>
        <album id="124" name="专辑" artist="艺术家" coverArt="124" songCount="10"/>
        <song id="125" title="歌曲" artist="艺术家" album="专辑" duration="240"/>
    </starred2>
</subsonic-response>
```