# getUser
返回用户详情。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `username` | 是 | 字符串 | 用户名 |

**响应：**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <user username="用户" email="user@example.com" admin="false"
          scrobblingEnabled="true" maxBitRate="320"
          downloadRole="true" uploadRole="false"
          playlistRole="true" coverArtRole="true"
          commentRole="false" podcastRole="false"
          shareRole="true" videoConversionRole="false"/>
</subsonic-response>
```