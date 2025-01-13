use axum::{
    response::{IntoResponse, Response},
    Json,
    http::StatusCode,
};
use crate::errors::YouTubeError;
use serde_json::json;

// Add this new struct for API errors
#[derive(Debug)]
pub enum ApiError {
    YouTubeError(YouTubeError),
    InvalidRequest(String),
    NotFound(String),
}

// Implement From conversion
impl From<YouTubeError> for ApiError {
    fn from(err: YouTubeError) -> Self {
        ApiError::YouTubeError(err)
    }
}

// Implement IntoResponse for ApiError instead of YouTubeError
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::YouTubeError(err) => {
                let (status, error_code, message) = match err {
                    YouTubeError::NotFound => (StatusCode::NOT_FOUND, "not_found", "Not found"),
                    YouTubeError::Ratelimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited", "Rate limited"),
                    YouTubeError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "Unauthorized"),
                    YouTubeError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", "Forbidden"),
                    YouTubeError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "internal_server_error", "Internal server error"),
                    YouTubeError::AccountClosed => (StatusCode::GONE, "account_closed", "Account is closed"),
                    YouTubeError::AccountTerminated => (StatusCode::GONE, "account_terminated", "Account is terminated"),
                    YouTubeError::SubscriptionsPrivate => (StatusCode::FORBIDDEN, "subscriptions_private", "Subscriptions are private"),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "unknown_error", "Unknown error occurred"),
                };

                (status, Json(json!({
                    "error": error_code,
                    "message": message
                }))).into_response()
            },
            ApiError::InvalidRequest(msg) => {
                (StatusCode::BAD_REQUEST, Json(json!({
                    "error": "invalid_request",
                    "message": msg
                }))).into_response()
            },
            ApiError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, Json(json!({
                    "error": "not_found",
                    "message": msg
                }))).into_response()
            }
        }
    }
}