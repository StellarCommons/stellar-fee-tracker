use std::error::Error;
use std::fmt;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Unified application error.
///
/// This ensures all layers (config, network, parsing)
/// fail in a predictable and debuggable way.
#[derive(Debug)]
pub enum AppError {
    Config(String),
    Network(String),
    Parse(String),
    Unknown(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Config(msg) => write!(f, "Config error: {}", msg),
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AppError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Network(_) => StatusCode::BAD_GATEWAY,
            AppError::Parse(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({ "error": self.to_string() }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    fn status_of(err: AppError) -> StatusCode {
        let response = err.into_response();
        response.status()
    }

    #[test]
    fn config_error_returns_500() {
        assert_eq!(
            status_of(AppError::Config("bad config".into())),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn network_error_returns_502() {
        assert_eq!(
            status_of(AppError::Network("timeout".into())),
            StatusCode::BAD_GATEWAY
        );
    }

    #[test]
    fn parse_error_returns_422() {
        assert_eq!(
            status_of(AppError::Parse("invalid json".into())),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn unknown_error_returns_500() {
        assert_eq!(
            status_of(AppError::Unknown("something broke".into())),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn display_messages_are_prefixed() {
        assert_eq!(
            AppError::Config("missing key".into()).to_string(),
            "Config error: missing key"
        );
        assert_eq!(
            AppError::Network("refused".into()).to_string(),
            "Network error: refused"
        );
        assert_eq!(
            AppError::Parse("bad field".into()).to_string(),
            "Parse error: bad field"
        );
        assert_eq!(
            AppError::Unknown("???".into()).to_string(),
            "Unknown error: ???"
        );
    }
}
