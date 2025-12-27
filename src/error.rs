//! 错误处理模块

use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::fmt;

/// 应用错误类型
#[derive(Debug)]
pub enum AppError {
    // Subsonic 错误码
    MissingParameter(String),
    AuthFailed(String),
    AccessDenied(String),
    NotFound(String),
    ServerBusy(String),

    // 内部错误
    DatabaseError(sqlx::Error),
    IoError(std::io::Error),
    AuthError(anyhow::Error),
    ValidationError(String),
    ConfigError(String),
}

/// Subsonic 错误响应格式
#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: String,
    version: String,
    error: SubsonicErrorPayload,
}

#[derive(Debug, Serialize)]
struct SubsonicErrorPayload {
    code: i32,
    message: String,
}

// 实现 Display trait
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MissingParameter(msg) => write!(f, "Missing parameter: {}", msg),
            AppError::AuthFailed(msg) => write!(f, "Authentication failed: {}", msg),
            AppError::AccessDenied(msg) => write!(f, "Access denied: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::ServerBusy(msg) => write!(f, "Server busy: {}", msg),
            AppError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AppError::IoError(err) => write!(f, "IO error: {}", err),
            AppError::AuthError(err) => write!(f, "Auth error: {}", err),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::ConfigError(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

// 实现 std::error::Error trait
impl std::error::Error for AppError {}

// 转换为 HTTP 响应
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_code, message) = match self {
            AppError::MissingParameter(msg) => (StatusCode::BAD_REQUEST, 10, msg),
            AppError::AuthFailed(msg) => (StatusCode::UNAUTHORIZED, 30, msg),
            AppError::AccessDenied(msg) => (StatusCode::FORBIDDEN, 40, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, 50, msg),
            AppError::ServerBusy(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 60, msg),
            AppError::DatabaseError(err) => {
                tracing::error!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, 0, "Database error".to_string())
            }
            AppError::IoError(err) => {
                tracing::error!("IO error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, 0, "IO error".to_string())
            }
            AppError::AuthError(err) => {
                tracing::error!("Auth error: {}", err);
                (StatusCode::UNAUTHORIZED, 30, "Authentication error".to_string())
            }
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, 0, msg),
            AppError::ConfigError(msg) => {
                tracing::error!("Config error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, 0, msg)
            }
        };

        let error_response = ErrorResponse {
            status: "failed".to_string(),
            version: "1.16.1".to_string(),
            error: SubsonicErrorPayload {
                code: error_code,
                message,
            },
        };

        (status_code, Json(error_response)).into_response()
    }
}

// From 实现，方便错误转换
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::AuthError(err)
    }
}

impl From<dotenvy::Error> for AppError {
    fn from(err: dotenvy::Error) -> Self {
        AppError::ConfigError(format!("Environment variable error: {}", err))
    }
}

// 便捷构造函数
impl AppError {
    pub fn missing_parameter(param: &str) -> Self {
        AppError::MissingParameter(format!("Required parameter '{}' is missing", param))
    }

    pub fn auth_failed(msg: &str) -> Self {
        AppError::AuthFailed(msg.to_string())
    }

    pub fn access_denied(msg: &str) -> Self {
        AppError::AccessDenied(msg.to_string())
    }

    pub fn not_found(resource: &str) -> Self {
        AppError::NotFound(format!("{} not found", resource))
    }

    pub fn server_busy(msg: &str) -> Self {
        AppError::ServerBusy(msg.to_string())
    }

    pub fn validation_error(msg: &str) -> Self {
        AppError::ValidationError(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::missing_parameter("username");
        assert_eq!(format!("{}", err), "Missing parameter: Required parameter 'username' is missing");
    }

    #[test]
    fn test_error_construction() {
        let err = AppError::auth_failed("Invalid credentials");
        match err {
            AppError::AuthFailed(msg) => assert_eq!(msg, "Invalid credentials"),
            _ => panic!("Wrong error type"),
        }
    }
}