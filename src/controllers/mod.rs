use crate::views::{InternalServerErrorTemplate, NotFoundTemplate};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub mod pastes;

pub async fn not_found() -> Result<(), Error> {
    Err(Error::NotFound)
}

pub async fn internal_server_error() -> Result<(), Error> {
    Err(Error::Generic)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("generic error")]
    Generic,

    #[error(transparent)]
    Database(#[from] tokio_rusqlite::Error),

    #[error("resource not found")]
    NotFound,
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Generic | Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn template(&self) -> ErrorTemplate {
        match self {
            Error::Generic | Error::Database(_) => {
                ErrorTemplate::InternalServerError(InternalServerErrorTemplate {})
            }
            Error::NotFound => ErrorTemplate::NotFound(NotFoundTemplate {}),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.status_code(), self.template()).into_response()
    }
}

enum ErrorTemplate {
    NotFound(NotFoundTemplate),
    InternalServerError(InternalServerErrorTemplate),
}

impl IntoResponse for ErrorTemplate {
    fn into_response(self) -> Response {
        match self {
            ErrorTemplate::InternalServerError(template) => template.into_response(),
            ErrorTemplate::NotFound(template) => template.into_response(),
        }
    }
}
