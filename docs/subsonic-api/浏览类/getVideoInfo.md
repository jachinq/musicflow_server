# getVideoInfo
返回视频详情。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 视频 ID |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <videoInfo id="123" title="视频标题">
        <captions id="124" label="英语"/>
    </videoInfo>
</subsonic-response>
```