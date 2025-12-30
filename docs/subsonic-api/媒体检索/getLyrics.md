# getLyrics
返回歌曲歌词。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `artist` | 否 | 字符串 | 艺术家名称 |
| `title` | 否 | 字符串 | 歌曲标题 |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
	<lyrics artist="艺术家" title="歌曲">歌词文本...</lyrics>
</subsonic-response>
```