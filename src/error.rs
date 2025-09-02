use actix_web::{HttpResponse, error::ResponseError, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct ErrorBody<'a> {
    pub error: ErrorEnvelope<'a>,
}

#[derive(Debug, Serialize)]
pub struct ErrorEnvelope<'a> {
    pub code: &'a str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Note not found")]
    NotFound,
    #[error("Internal server error")]
    InternalError,
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let (code, msg, details) = match self {
            ApiError::NotFound => ("NOT_FOUND", "Note not found".to_string(), None),
            ApiError::InternalError => ("INTERNAL", "Internal server error".to_string(), None),
            ApiError::BadRequest(m) => ("BAD_REQUEST", m.clone(), None),
        };

        HttpResponse::build(self.status_code()).json(ErrorBody {
            error: ErrorEnvelope {
                code,
                message: msg,
                details,
            },
        })
    }
}
