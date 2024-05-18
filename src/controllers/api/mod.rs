use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

pub mod pastes;

pub async fn not_found() -> Result<(), self::Error> {
    Err(self::Error::NotFound)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("malformed request")]
    BadRequest(#[from] validator::ValidationErrors),

    #[error("invalid authentication credentials")]
    Unauthorized,

    #[error("resource not found")]
    NotFound,

    #[allow(clippy::enum_variant_names)]
    #[error("internal server error: {0}")]
    InternalServerError(#[from] Box<dyn std::error::Error>),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Error::BadRequest(_err) => (
                StatusCode::BAD_REQUEST,
                ErrorBody::new(StatusCode::BAD_REQUEST, "Invalid request parameters."),
            ),

            Error::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                ErrorBody::new(
                    StatusCode::UNAUTHORIZED,
                    "Invalid authentication credentials.",
                ),
            ),

            Error::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorBody::new(StatusCode::NOT_FOUND, "Resource not found."),
            ),

            Error::InternalServerError(err) => {
                tracing::error!(%err, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorBody::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An unexpected error occurred.",
                    ),
                )
            }
        };

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
    fn new(status_code: StatusCode, message: &str) -> ErrorBody {
        let status = status_code.as_u16();
        let error = status_code
            .canonical_reason()
            .unwrap_or_default()
            .to_string();
        let message = message.to_string();
        ErrorBody {
            status,
            error,
            message,
        }
    }
}
