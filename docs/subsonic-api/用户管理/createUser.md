# createUser
创建新用户。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `username` | 是 | 字符串 | 用户名 |
| `password` | 是 | 字符串 | 密码 |
| `email` | 是 | 字符串 | 邮箱地址 |
| `ldapAuth` | 否 | 布尔值 | 使用 LDAP 认证 |
| `admin` | 否 | 布尔值 | 管理员权限 |
| `scrobblingEnabled` | 否 | 布尔值 | 启用播放记录 |
| `maxBitRate` | 否 | 整数 | 最大比特率 |
| `downloadRole` | 否 | 布尔值 | 下载权限 |
| `uploadRole` | 否 | 布尔值 | 上传权限 |
| `playlistRole` | 否 | 布尔值 | 播放列表管理 |
| `coverArtRole` | 否 | 布尔值 | 封面管理 |
| `commentRole` | 否 | 布尔值 | 评论权限 |
| `podcastRole` | 否 | 布尔值 | 播客管理 |
| `shareRole` | 否 | 布尔值 | 分享权限 |
| `videoConversionRole` | 否 | 布尔值 | 视频转换 |

**响应：**
成功/失败状态