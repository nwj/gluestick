use crate::models::prelude::Error as ModelsError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("malformed request")]
    BadRequest(Box<dyn std::error::Error>),

    #[error("invalid authentication credentials")]
    Unauthorized,

    #[error("insufficient privileges")]
    Forbidden,

    #[error("resource not found")]
    NotFound,

    #[error("internal server error: {0}")]
    InternalServerError(Box<dyn std::error::Error>),
}

impl From<ModelsError> for Error {
    fn from(error: ModelsError) -> Self {
        Self::InternalServerError(Box::new(error))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Error::BadRequest(err) => (StatusCode::BAD_REQUEST, format!("{err}")),

            Error::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Invalid authentication credentials.".into(),
            ),

            Error::Forbidden => (StatusCode::FORBIDDEN, "Insufficient privileges".into()),

            Error::NotFound => (StatusCode::NOT_FOUND, "Resource not found.".into()),

            Error::InternalServerError(err) => {
                tracing::error!(%err, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected error occurred.".into(),
                )
            }
        };

        let body = ErrorBody::new(status, &message);
        (status, Json(body)).into_response()
    }
}

#[derive(Serialize)]
struct ErrorBody {
    status: u16,
    error: String,
    message: String,
}

impl ErrorBody {
    fn new(status_code: StatusCode, message: &str) -> Self {
        Self {
            status: status_code.as_u16(),
            error: status_code
                .canonical_reason()
                .unwrap_or_default()
                .to_string(),
            message: message.to_string(),
        }
    }
}
