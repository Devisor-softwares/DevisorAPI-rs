use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{error, info, warn, debug};

/// Custom error types for better error handling
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl ApiError {
    /// Get the appropriate HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Auth(_) => StatusCode::UNAUTHORIZED,
            ApiError::Validation(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Get error category for logging
    pub fn category(&self) -> &'static str {
        match self {
            ApiError::Database(_) => "database",
            ApiError::Auth(_) => "authentication",
            ApiError::Validation(_) => "validation",
            ApiError::NotFound(_) => "not_found",
            ApiError::Conflict(_) => "conflict",
            ApiError::Forbidden(_) => "forbidden",
            ApiError::Internal(_) => "internal",
            ApiError::BadRequest(_) => "bad_request",
            ApiError::Unauthorized(_) => "unauthorized",
            ApiError::ServiceUnavailable(_) => "service_unavailable",
        }
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::Database(_) => "DATABASE_ERROR",
            ApiError::Auth(_) => "AUTH_ERROR",
            ApiError::Validation(_) => "VALIDATION_ERROR",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::Conflict(_) => "CONFLICT",
            ApiError::Forbidden(_) => "FORBIDDEN",
            ApiError::Internal(_) => "INTERNAL_ERROR",
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::Unauthorized(_) => "UNAUTHORIZED",
            ApiError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        let category = self.category();
        let message = self.to_string();

        // Log the error with appropriate level
        match status {
            StatusCode::INTERNAL_SERVER_ERROR => {
                error!(
                    category = category,
                    error_code = error_code,
                    message = message,
                    "Internal server error occurred"
                );
            }
            StatusCode::BAD_REQUEST | StatusCode::VALIDATION_ERROR => {
                warn!(
                    category = category,
                    error_code = error_code,
                    message = message,
                    "Client error occurred"
                );
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                warn!(
                    category = category,
                    error_code = error_code,
                    message = message,
                    "Authorization error occurred"
                );
            }
            StatusCode::NOT_FOUND => {
                debug!(
                    category = category,
                    error_code = error_code,
                    message = message,
                    "Resource not found"
                );
            }
            _ => {
                info!(
                    category = category,
                    error_code = error_code,
                    message = message,
                    "API error occurred"
                );
            }
        }

        let error_response = json!({
            "success": false,
            "error": {
                "code": error_code,
                "message": message,
                "category": category
            },
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "path": "" // This will be filled by middleware
        });

        (status, Json(error_response)).into_response()
    }
}

/// Custom success response builder
#[derive(Debug)]
pub struct ApiResponse {
    data: Option<Value>,
    message: Option<String>,
    meta: Option<HashMap<String, Value>>,
}

impl ApiResponse {
    pub fn new() -> Self {
        Self {
            data: None,
            message: None,
            meta: None,
        }
    }

    pub fn data<T: serde::Serialize>(mut self, data: T) -> Self {
        self.data = Some(serde_json::to_value(data).unwrap_or(Value::Null));
        self
    }

    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn meta<K: Into<String>, V: serde::Serialize>(mut self, key: K, value: V) -> Self {
        let mut meta = self.meta.take().unwrap_or_default();
        meta.insert(key.into(), serde_json::to_value(value).unwrap_or(Value::Null));
        self.meta = Some(meta);
        self
    }

    pub fn success() -> Self {
        Self::new().message("Operation successful")
    }

    pub fn created() -> Self {
        Self::new().message("Resource created successfully")
    }

    pub fn updated() -> Self {
        Self::new().message("Resource updated successfully")
    }

    pub fn deleted() -> Self {
        Self::new().message("Resource deleted successfully")
    }
}

impl Default for ApiResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        let response = json!({
            "success": true,
            "data": self.data,
            "message": self.message,
            "meta": self.meta,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        info!(
            success = true,
            message = self.message.unwrap_or_default(),
            "API response sent"
        );

        (StatusCode::OK, Json(response)).into_response()
    }
}

/// Middleware to add request path to error responses
pub async fn error_path_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    let response = next.run(request).await;

    // If this is an error response, try to add the path
    if response.status().is_client_error() || response.status().is_server_error() {
        // Note: This is a simplified approach. In a real implementation,
        // you might want to use a custom response wrapper or extract
        // and modify the JSON body more carefully.
        debug!(
            method = method,
            path = path,
            status = response.status().as_u16(),
            "Error response sent"
        );
    } else {
        debug!(
            method = method,
            path = path,
            status = response.status().as_u16(),
            "Success response sent"
        );
    }

    Ok(response)
}

/// Request logging middleware
pub async fn request_logging_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    info!(
        method = method,
        path = path,
        user_agent = user_agent,
        "Request started"
    );

    let response = next.run(request).await;

    let duration = start_time.elapsed();
    let status = response.status().as_u16();

    info!(
        method = method,
        path = path,
        status = status,
        duration_ms = duration.as_millis(),
        user_agent = user_agent,
        "Request completed"
    );

    Ok(response)
}

/// Convenience functions for common error responses
pub mod error_responses {
    use super::*;

    pub fn database_error(err: impl Into<String>) -> ApiError {
        ApiError::Database(sqlx::Error::Protocol(format!("Database error: {}", err.into())))
    }

    pub fn auth_error(message: impl Into<String>) -> ApiError {
        ApiError::Auth(message.into())
    }

    pub fn validation_error(message: impl Into<String>) -> ApiError {
        ApiError::Validation(message.into())
    }

    pub fn not_found(resource: impl Into<String>) -> ApiError {
        ApiError::NotFound(format!("{} not found", resource.into()))
    }

    pub fn conflict(message: impl Into<String>) -> ApiError {
        ApiError::Conflict(message.into())
    }

    pub fn forbidden(message: impl Into<String>) -> ApiError {
        ApiError::Forbidden(message.into())
    }

    pub fn internal_error(message: impl Into<String>) -> ApiError {
        ApiError::Internal(message.into())
    }

    pub fn bad_request(message: impl Into<String>) -> ApiError {
        ApiError::BadRequest(message.into())
    }

    pub fn unauthorized(message: impl Into<String>) -> ApiError {
        ApiError::Unauthorized(message.into())
    }

    pub fn service_unavailable(message: impl Into<String>) -> ApiError {
        ApiError::ServiceUnavailable(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_status_codes() {
        assert_eq!(ApiError::NotFound("test".to_string()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ApiError::Auth("test".to_string()).status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(ApiError::Validation("test".to_string()).status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_api_error_codes() {
        assert_eq!(ApiError::NotFound("test".to_string()).error_code(), "NOT_FOUND");
        assert_eq!(ApiError::Auth("test".to_string()).error_code(), "AUTH_ERROR");
        assert_eq!(ApiError::Validation("test".to_string()).error_code(), "VALIDATION_ERROR");
    }

    #[test]
    fn test_api_response_builder() {
        let response = ApiResponse::new()
            .message("Test message")
            .meta("version", "1.0")
            .data(json!({"key": "value"}));

        assert_eq!(response.message, Some("Test message".to_string()));
        assert!(response.meta.is_some());
        assert!(response.data.is_some());
    }
}
