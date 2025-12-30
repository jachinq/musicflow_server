# updateUser

修改现有的 Subsonic 用户，使用以下参数：

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `username` | 是 | 字符串 | 用户名 |
| `password` | 否 | 字符串 | 新密码 |
| `email` | 否 | 字符串 | 新电子邮件地址 |
| `ldapAuthenticated` | 否 | 字符串 | 是否通过 LDAP 认证 |
| `adminRole` | 否 | 字符串 | 是否为管理员 |
| `settingsRole` | 否 | 字符串 | 是否有设置权限 |
| `streamRole` | 否 | 字符串 | 是否有流媒体权限 |
| `jukeboxRole` | 否 | 字符串 | 是否有 jukebox 权限 |
| `downloadRole` | 否 | 字符串 | 是否有下载权限 |
| `uploadRole` | 否 | 字符串 | 是否有上传权限 |
| `coverArtRole` | 否 | 字符串 | 是否有封面权限 |
| `commentRole` | 否 | 字符串 | 是否有评论权限 |
| `podcastRole` | 否 | 字符串 | 是否有播客权限 |
| `shareRole` | 否 | 字符串 | 是否有分享权限 |
| `videoConversionRole` | 否 | 字符串 | 是否有视频转换权限 |
| `musicFolderId` | 否 | 字符串 | 音乐文件夹 ID |
| `maxBitRate` | 否 | 整数 | 最大比特率 | 

**响应：**

成功/失败状态