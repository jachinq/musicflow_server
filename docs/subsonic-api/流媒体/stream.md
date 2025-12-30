# stream

流式传输歌曲。

**参数：**
| 参数 | 必需 | 类型 | 描述 |
|------|------|------|------|
| `id` | 是 | 字符串 | 歌曲 ID |
| `maxBitRate` | 否 | 整数 | 最大比特率 |
| `format` | 否 | 字符串 | 输出格式：`mp3`、`flac`、`wav`、`aac`、`m4a`、`opus`、`oga` |
| `timeOffset` | 否 | 整数 | 时间偏移（秒） |
| `size` | 否 | 字符串 | 视频尺寸（用于视频流） |
| `estimateContentLength` | 否 | 布尔值 | 估算内容长度 |
| `converted` | 否 | 布尔值 | 实时转换 |

**响应：**
二进制音频/视频数据