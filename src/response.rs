//! 统一响应包装器
//!
//! 提供自动根据格式序列化的响应类型

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::models::response::{ResponseFormat, SubsonicResponse, ToXml};

/// 自动根据格式序列化的响应包装器
///
/// # 使用示例
///
/// ```rust
/// use crate::response::ApiResponse;
/// use crate::extractors::Format;
///
/// async fn handler(Format(format): Format) -> ApiResponse<Data> {
///     let data = get_data();
///     ApiResponse::ok(Some(data), format)
/// }
/// ```
pub struct ApiResponse<T> {
    data: SubsonicResponse<T>,
    format: ResponseFormat,
}

impl<T> ApiResponse<T> {
    /// 创建新的 ApiResponse
    pub fn new(data: SubsonicResponse<T>, format: ResponseFormat) -> Self {
        Self { data, format }
    }

    /// 便捷构造函数 - 成功响应
    pub fn ok(data: Option<T>, format: ResponseFormat) -> Self {
        let response = match format {
            ResponseFormat::Json => SubsonicResponse::ok(data),
            ResponseFormat::Xml => SubsonicResponse::ok_xml(data),
        };
        Self::new(response, format)
    }

    /// 便捷构造函数 - 错误响应
    pub fn _error(code: i32, message: String, format: ResponseFormat) -> Self {
        let response = match format {
            ResponseFormat::Json => SubsonicResponse::failed(code, message),
            ResponseFormat::Xml => SubsonicResponse::failed_xml(code, message),
        };
        Self::new(response, format)
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize + ToXml + Clone,
{
    fn into_response(self) -> Response {
        match self.format {
            ResponseFormat::Json => {
                // 序列化为 JSON
                let json = match serde_json::to_string(&self.data) {
                    Ok(json) => json,
                    Err(err) => {
                        tracing::error!("JSON serialization error: {}", err);
                        return (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error")
                            .into_response();
                    }
                };

                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, self.format.content_type())],
                    json,
                )
                    .into_response()
            }
            ResponseFormat::Xml => {
                // 序列化为 XML (使用手动构建)
                let xml = self.data.to_xml();

                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, self.format.content_type())],
                    xml,
                )
                    .into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_json_response() {
        let response = ApiResponse::ok(None::<()>, ResponseFormat::Json);
        let http_response = response.into_response();

        assert_eq!(http_response.status(), StatusCode::OK);
        assert_eq!(
            http_response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json; charset=utf-8"
        );
    }

    #[tokio::test]
    async fn test_xml_response() {
        let response = ApiResponse::ok(None::<()>, ResponseFormat::Xml);
        let http_response = response.into_response();

        assert_eq!(http_response.status(), StatusCode::OK);
        assert_eq!(
            http_response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/xml; charset=utf-8"
        );
    }
}
