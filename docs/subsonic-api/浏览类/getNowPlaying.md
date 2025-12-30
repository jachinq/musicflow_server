# getNowPlaying
返回正在播放的歌曲。

**参数：**
无

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <nowPlaying>
        <entry id="123" title="歌曲" artist="艺术家" username="用户" minutesAgo="5"/>
    </nowPlaying>
</subsonic-response>
```