# getChatMessages
返回聊天消息。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `since` | 否 | Long | Unix 时间戳（毫秒） |

**响应：**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="1.16.1">
    <chatMessages>
        <message id="123" username="用户" message="你好" time="1234567890"/>
    </chatMessages>
</subsonic-response>
```