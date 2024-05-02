use crate::views::{InternalServerErrorTemplate, NotFoundTemplate};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub mod pastes;

pub async fn not_found() -> Result<(), Error> {
    Err(Error::NotFound)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] tokio_rusqlite::Error),

    #[error("resource not found")]
    NotFound,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, template) = match self {
            Error::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorTemplate::NotFound(NotFoundTemplate {}),
            ),

            Error::Database(err) => {
                tracing::error!(%err, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorTemplate::InternalServerError(InternalServerErrorTemplate {}),
                )
            }
        };

        (status, template).into_response()
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
