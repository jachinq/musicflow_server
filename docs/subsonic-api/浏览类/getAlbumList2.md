# getAlbumList2
返回包含更多详情的专辑列表。

**参数：**
与 getAlbumList 相同。

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <albumList2>
        <album id="123" name="专辑名称" artist="艺术家名称" artistId="456"
            coverArt="123" songCount="10" created="2020-01-01T00:00:00Z"
            duration="3600" playCount="100" year="2020" genre="摇滚"/>
    </albumList2>
</subsonic-response>
```