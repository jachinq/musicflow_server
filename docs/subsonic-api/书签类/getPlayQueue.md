# getPlayQueue

返回当前用户的播放队列状态（由 savePlayQueue 设置）。播放队列中包含的歌曲、当前播放的歌曲以及当前歌曲中的位置。通常用于允许用户在不同的客户端/应用之间切换，同时保留播放队列（例如在听音乐书时）。

API: `/rest/getPlayQueue`

**参数：**

无

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.12.0">
  <playQueue current="133" position="45000" username="admin" changed="2015-02-18T15:22:22.825Z" changedBy="android">
    <entry id="132" parent="131" isDir="false" title="These Are Days" album="MTV Unplugged" artist="10,000 Maniacs" track="1" year="1993" genre="Soft Rock" coverArt="131" size="5872262" contentType="audio/mpeg" suffix="mp3" duration="293" bitRate="160" path="10,000 Maniacs/MTV Unplugged/01 - These Are Days.mp3" isVideo="false" created="2004-10-25T20:36:03.000Z" albumId="0" artistId="0" type="music"/>
    <entry id="133" parent="131" isDir="false" title="Eat For Two" album="MTV Unplugged" artist="10,000 Maniacs" track="2" year="1993" genre="Soft Rock" coverArt="131" size="5253248" contentType="audio/mpeg" suffix="mp3" duration="262" bitRate="160" path="10,000 Maniacs/MTV Unplugged/02 - Eat For Two.mp3" isVideo="false" created="2004-10-25T20:36:06.000Z" albumId="0" artistId="0" type="music"/>
  </playQueue>
</subsonic-response>
```