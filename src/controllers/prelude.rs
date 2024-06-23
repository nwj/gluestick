use crate::models::prelude::Error as ModelsError;
use crate::views::{InternalServerErrorTemplate, NotFoundTemplate};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

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

    #[allow(clippy::enum_variant_names)]
    #[error("internal server error: {0}")]
    InternalServerError(Box<dyn std::error::Error>),
}

impl From<ModelsError> for Error {
    fn from(error: ModelsError) -> Self {
        match error {
            ModelsError::Validation(_) | ModelsError::ParseInt(_) => {
                Self::BadRequest(Box::new(error))
            }
            _ => Self::InternalServerError(Box::new(error)),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::BadRequest(err) => {
                tracing::error!(%err, "bad request");
                (StatusCode::BAD_REQUEST, ()).into_response()
            }

            Error::Unauthorized => (StatusCode::UNAUTHORIZED, ()).into_response(),

            Error::Forbidden => (StatusCode::FORBIDDEN, ()).into_response(),

            Error::NotFound => {
                (StatusCode::NOT_FOUND, NotFoundTemplate { session: None }).into_response()
            }

            Error::InternalServerError(err) => {
                tracing::error!(%err, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    InternalServerErrorTemplate { session: None },
                )
                    .into_response()
            }
        }
    }
}
