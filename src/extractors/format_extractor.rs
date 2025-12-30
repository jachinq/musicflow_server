//! 响应格式提取器
//!
//! 从请求中提取客户端期望的响应格式 (JSON/XML)
//! 支持两种方式:
//! 1. 查询参数 f=json 或 f=xml (Subsonic API 标准)
//! 2. Accept 请求头

use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use serde::Deserialize;

use crate::models::response::ResponseFormat;

/// 格式查询参数
#[derive(Deserialize)]
struct FormatParam {
    #[serde(rename = "f")]
    format: Option<String>,
}

/// 响应格式提取器
///
/// # 使用示例
///
/// ```rust
/// use crate::extractors::Format;
///
/// async fn handler(Format(format): Format) -> ApiResponse<Data> {
///     ApiResponse::ok(Some(data), format)
/// }
/// ```
pub struct Format(pub ResponseFormat);

#[async_trait]
impl<S> FromRequestParts<S> for Format
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. 优先从查询参数 f 获取 (Subsonic API 标准)
        if let Ok(Query(param)) = Query::<FormatParam>::from_request_parts(parts, state).await {
            if let Some(format_str) = param.format {
                let format = ResponseFormat::parse_str(&format_str);
                tracing::debug!("Format from query parameter: {:?}", format);
                return Ok(Format(format));
            }
        }

        // 2. 其次检查 Accept 请求头
        if let Some(accept) = parts.headers.get("accept") {
            if let Ok(accept_str) = accept.to_str() {
                if accept_str.contains("application/xml") || accept_str.contains("text/xml") {
                    tracing::debug!("Format from Accept header: XML");
                    return Ok(Format(ResponseFormat::Xml));
                }
            }
        }

        // 3. 默认返回 JSON
        tracing::debug!("Format: default XML");
        Ok(Format(ResponseFormat::Xml))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[tokio::test]
    async fn test_format_from_query_param() {
        let req = Request::builder()
            .uri("http://localhost/test?f=xml")
            .body(())
            .unwrap();

        let (mut parts, _) = req.into_parts();
        let Format(format) = Format::from_request_parts(&mut parts, &()).await.unwrap();

        assert_eq!(format, ResponseFormat::Xml);
    }

    #[tokio::test]
    async fn test_format_default() {
        let req = Request::builder()
            .uri("http://localhost/test")
            .body(())
            .unwrap();

        let (mut parts, _) = req.into_parts();
        let Format(format) = Format::from_request_parts(&mut parts, &()).await.unwrap();

        assert_eq!(format, ResponseFormat::Json);
    }
}
