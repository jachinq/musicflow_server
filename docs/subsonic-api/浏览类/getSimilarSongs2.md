# getRandomSongs2

返回给定艺术家及相似艺术家的随机歌曲集合，使用来自 last.fm 的数据。通常用于艺术家广播功能。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 艺术家/专辑/歌曲 ID |
| `count` | 否 | 整数 | 歌曲数量，默认值为 50 |

**响应：**
```xml
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.11.0">
  <similarSongs>
    <song id="1631" parent="1628" isDir="false" title="A Whiter Shade Of Pale" album="Medusa" artist="Annie Lennox" track="3" coverArt="1628" size="5068173" contentType="audio/mpeg" suffix="mp3" duration="316" bitRate="128" path="Annie Lennox/Medusa/03 - A Whiter Shade Of Pale.MP3" isVideo="false" created="2004-11-08T22:21:17.000Z" albumId="471" artistId="305" type="music"/>
    <song id="4654" parent="4643" isDir="false" title="somebodys somebody" album="christina aguilera" artist="christina aguilera" track="8" year="1999" genre="Pop" coverArt="4643" size="6039760" contentType="audio/mpeg" suffix="mp3" duration="302" bitRate="160" path="Christina Aguilera/Album/08-cps-christina_aguilera--somebodys_somebody.mp3" isVideo="false" created="2004-11-25T22:18:53.000Z" albumId="698" type="music"/>
  </similarSongs>
</subsonic-response>
```