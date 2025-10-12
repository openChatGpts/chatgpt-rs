use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::utils::ChatGptError;
use super::types::ErrorResponse;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl From<ChatGptError> for ApiError {
    fn from(err: ChatGptError) -> Self {
        let status = match err {
            ChatGptError::InvalidProxy(_) => StatusCode::BAD_REQUEST,
            ChatGptError::Authentication(_) => StatusCode::UNAUTHORIZED,
            ChatGptError::IpFlagged => StatusCode::FORBIDDEN,
            ChatGptError::Network(_) => StatusCode::BAD_GATEWAY,
            ChatGptError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ChatGptError::ChallengeSolve(_)
            | ChatGptError::VmExecution(_)
            | ChatGptError::InvalidResponse(_)
            | ChatGptError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ChatGptError::Json(_)
            | ChatGptError::Base64Decode(_)
            | ChatGptError::Io(_)
            | ChatGptError::Image(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };

        Self::new(status, err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let payload = Json(ErrorResponse {
            status: "error",
            detail: self.message,
        });

        (self.status, payload).into_response()
    }
}
