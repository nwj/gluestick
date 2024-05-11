use crate::{
    models,
    views::{InternalServerErrorTemplate, NotFoundTemplate},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub mod api;
pub mod misc;
pub mod pastes;
pub mod sessions;
pub mod users;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] tokio_rusqlite::Error),

    #[error(transparent)]
    User(#[from] models::user::Error),

    #[error(transparent)]
    Validation(#[from] validator::ValidationErrors),

    #[error("resource not found")]
    NotFound,

    #[error("invalid credentials")]
    Unauthorized,

    #[error(transparent)]
    PasswordHash(#[from] argon2::password_hash::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, template) = match self {
            Error::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorTemplate::NotFound(NotFoundTemplate {}),
            ),

            Error::Unauthorized => (StatusCode::UNAUTHORIZED, ErrorTemplate::Blank),

            Error::Validation(err) => {
                tracing::error!(%err, "test");
                (StatusCode::BAD_REQUEST, ErrorTemplate::Blank)
            }

            Error::Database(err) => {
                tracing::error!(%err, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorTemplate::InternalServerError(InternalServerErrorTemplate {}),
                )
            }

            Error::User(err) => {
                tracing::error!(%err, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorTemplate::InternalServerError(InternalServerErrorTemplate {}),
                )
            }

            Error::PasswordHash(err) => {
                tracing::error!(%err, "hashing error");
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
    Blank,
}

impl IntoResponse for ErrorTemplate {
    fn into_response(self) -> Response {
        match self {
            ErrorTemplate::InternalServerError(template) => template.into_response(),
            ErrorTemplate::NotFound(template) => template.into_response(),
            ErrorTemplate::Blank => ().into_response(),
        }
    }
}
