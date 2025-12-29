//! 响应格式类型定义

/// 响应格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    /// JSON 格式 (默认)
    Json,
    /// XML 格式
    Xml,
}

impl Default for ResponseFormat {
    fn default() -> Self {
        Self::Xml
    }
}

impl ResponseFormat {
    /// 从字符串解析格式
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "xml" => Self::Xml,
            "json" | _ => Self::Json,
        }
    }

    /// 获取对应的 Content-Type
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Json => "application/json; charset=utf-8",
            Self::Xml => "application/xml; charset=utf-8",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_str() {
        assert_eq!(ResponseFormat::from_str("json"), ResponseFormat::Json);
        assert_eq!(ResponseFormat::from_str("JSON"), ResponseFormat::Json);
        assert_eq!(ResponseFormat::from_str("xml"), ResponseFormat::Xml);
        assert_eq!(ResponseFormat::from_str("XML"), ResponseFormat::Xml);
        assert_eq!(ResponseFormat::from_str("unknown"), ResponseFormat::Json);
    }

    #[test]
    fn test_content_type() {
        assert_eq!(
            ResponseFormat::Json.content_type(),
            "application/json; charset=utf-8"
        );
        assert_eq!(
            ResponseFormat::Xml.content_type(),
            "application/xml; charset=utf-8"
        );
    }
}
